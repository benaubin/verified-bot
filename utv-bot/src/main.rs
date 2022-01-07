mod handlers;

use std::collections::HashMap;
use std::{char, env};

use lazy_static::lazy_static;
use serde_json::Value;
use serenity::client::bridge::gateway::event::ShardStageUpdateEvent;
use serenity::model::channel::{
    Channel, ChannelCategory, GuildChannel, Message, PartialGuildChannel, Reaction, StageInstance,
};
use serenity::model::event::{
    ChannelPinsUpdateEvent, GuildMembersChunkEvent, InviteCreateEvent, InviteDeleteEvent,
    MessageUpdateEvent, PresenceUpdateEvent, ResumedEvent, ThreadListSyncEvent,
    ThreadMembersUpdateEvent, TypingStartEvent, VoiceServerUpdateEvent,
};
use serenity::model::gateway::{Activity, Presence};
use serenity::model::guild::{
    Emoji, Guild, GuildUnavailable, Integration, Member, PartialGuild, Role, ThreadMember,
};
use serenity::model::id::{
    ApplicationId, ChannelId, EmojiId, GuildId, IntegrationId, MessageId, RoleId,
};
use serenity::model::prelude::{CurrentUser, User, VoiceState};
use serenity::utils::Color;
use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    model::{
        event::GuildMemberUpdateEvent,
        gateway::Ready,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};
use serenity::futures::TryFutureExt;
use serenity::http::{CacheHttp, GuildPagination};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use sled::IVec;
use tokio::select_priv_declare_output_enum;

lazy_static! {
    static ref USERDB: sled::Db = sled::open("user_db").unwrap();
    static ref ROLEDB: sled::Db = sled::open("role_db").unwrap();
    static ref SHARED_KEY: Vec<u8> = {
        let key = std::env::var("SHARED_KEY").expect("SHARED_KEY env variable missing");
        base64::decode_config(key, base64::URL_SAFE_NO_PAD)
            .expect("Failed to decode base64 SHARED_KEY")
    };
}

struct Handler;

/// Scans all users in the guild to check nickname compliance
async fn scan(command: ApplicationCommandInteraction, guild: GuildId, ctx: Context) -> serenity::Result<()> {
    if !command.member.as_ref().unwrap().permissions.unwrap().administrator() {
        return command.create_interaction_response(&ctx.http, |interaction| {
            interaction
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|embed| {
                        embed
                            .title("You must be a guild admin to run this command.")
                    })
                })
        }).await;
    }
    let guild = ctx.http.get_guild(guild.into()).await?;
    for mut member in guild.members(&ctx.http, None, None).await? {
        handle_member_status(&ctx, &mut member).await;
    }
    command.create_interaction_response(&ctx.http, |interaction| {
        interaction
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.create_embed(|embed| {
                    embed
                        .title("Command Completed")
                })
            })
    }).await
}

/// Modifies the name and roles of the user to either sanitize it or assign it the ✓
async fn handle_member_status(ctx: &Context, mem: &mut Member) -> bool {
    let guild = ctx.http.get_guild(mem.guild_id.into()).await.unwrap();
    let verified_role = get_verified_role(&ctx, &guild).await;
    let key: u64 = mem.user.id.into();
    let key = key.to_be_bytes().to_vec();
    let original = mem.display_name().to_string();
    let mut cleaned = mem.display_name().replace("✓", "_");
    if let Ok(Some(_)) = USERDB.get(key) {
        // verified
        if !mem.roles.contains(&verified_role.id) {
            mem.add_role(&ctx.http, verified_role.id).await.unwrap();
        }
        if !original.ends_with("✓") {
            cleaned.push_str(" ✓");
        } else {
            return true;
        }
    }
    if original != cleaned {
        mem.edit(&ctx.http, |m| m.nickname(cleaned)).await;
        true
    } else {
        false
    }
}

/// Gets the Verified Role and Creates it if needed
async fn get_verified_role<'a>(ctx: &'a Context, guild: &'a PartialGuild) -> &'a Role {
    let key: u64 = guild.id.into();
    let key: Vec<u8> = key.to_be_bytes().to_vec();
    let role_id = match ROLEDB.get(&key).unwrap() {
        Some(value) => {
            let role_id = RoleId(u64::from_be_bytes(value.to_vec().as_slice().try_into().unwrap()));
            guild.roles.get(&role_id).unwrap().id
        },
        None => {
            let new_role = guild.create_role(&ctx.http, |r| {
                r
                    .name("UTexas Verified")
                    .hoist(true)
                    .mentionable(true)
                    .colour(0xbf5700)
            }).await.unwrap();
            ROLEDB.insert(key, new_role.id.as_u64().to_be_bytes().to_vec()).unwrap();
            new_role.id
        }
    };
    let role = guild.roles.get(&role_id).unwrap();
    role
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        for (_, mut member) in guild.members {
            handle_member_status(&ctx, &mut member).await;
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        handle_member_status(&ctx, &mut new_member).await;
    }

    async fn guild_member_update(&self, ctx: Context, update: GuildMemberUpdateEvent) {
        if let Some(new_nick) = update.nick {
            if let Ok(guild) = ctx.http.get_guild(update.guild_id.into()).await {
                if let Ok(mut member) = guild.member(&ctx.http, update.user.id).await {
                    handle_member_status(&ctx, &mut member).await;
                }
            }
        }
    }

    /// Handling token received in DM's
    async fn message(&self, ctx: Context, new_message: Message) {
        if let None = new_message.guild_id {
            let token = new_message.content.as_str();
            if let Ok(data) = utv_token::decode_token(token, &SHARED_KEY) {
                let key: u64 = new_message.author.id.into();
                let key = key.to_be_bytes().to_vec();
                new_message.reply(
                    &ctx.http,
                    match USERDB.get(&key) {
                        Ok(Some(_)) => "This Discord account has already been associated with a UT EID. Please contact `support@verifiedbot.com` if you believe this is a mistake.",
                        Ok(None) => {
                            USERDB
                                .insert(key, serde_json::to_vec(&data).expect("Failed to save verified user (Encoding error)"))
                                .expect("Failed to save verified user (DB error)");
                            ctx.set_activity(Activity::watching(format!("{} verified students", USERDB.len()))).await;
                            if let Ok(guildInfos) = ctx.http.get_guilds(&GuildPagination::After(GuildId(0)), 100).await {
                                for guildInfo in guildInfos {
                                    if let Ok(guild) = ctx.http.get_guild(guildInfo.id.into()).await {
                                        if let Ok(members) = guild.members(&ctx.http, None, None).await {
                                            for mut mem in members {
                                                if mem.user.id == new_message.author.id {
                                                    handle_member_status(&ctx, &mut mem).await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            "You've been verified! Please check participating servers to see a check mark next to your name!"
                        }
                        Err(_) => "Oh Snap! Something went wrong on our side. Please try again later."
                    }
                ).await;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::watching(format!("{} verified students", USERDB.len()))).await;

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("verify")
                        .description("Verify your Discord Account")
                        .create_option(|option| {
                            option
                                .name("eid")
                                .description("Your UT EID")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("help")
                        .description("Learn more about the bot and its commands")
                })
                .create_application_command(|command| {
                    command
                        .name("rescan")
                        .description("Check all users in the guild for nickname compliance")
                })
        })
        .await;

        println!(
            "I now have the following global slash commands: {:#?}",
            commands
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if let Err(why) = match command.data.name.as_str() {
                "verify" => handlers::verify(command, ctx).await,
                "rescan" => match command.guild_id {
                    Some(guild) => scan(command, guild, ctx).await,
                    None => {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.create_embed(|embed| {
                                            embed.title(
                                            "This command must be run inside of a guild, not a DM.",
                                        )
                                        })
                                    })
                            })
                            .await
                    }
                },
                _ => {
                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.create_embed(|embed| match command.data.name.as_str() {
                                        _ => handlers::unknown_command(embed, &command),
                                    })
                                })
                        })
                        .await
                }
            } {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord bot token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token)
        .intents(GatewayIntents::GUILD_MEMBERS | GatewayIntents::DIRECT_MESSAGES)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
