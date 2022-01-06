mod handlers;

use std::collections::HashMap;
use std::{char, env};

use lazy_static::lazy_static;
use serenity::client::bridge::gateway::event::ShardStageUpdateEvent;
use serenity::model::channel::{
    Channel, ChannelCategory, GuildChannel, Message, PartialGuildChannel, Reaction, StageInstance,
};
use serenity::model::event::{
    ChannelPinsUpdateEvent, GuildMembersChunkEvent, InviteCreateEvent, InviteDeleteEvent,
    MessageUpdateEvent, PresenceUpdateEvent, ResumedEvent, ThreadListSyncEvent,
    ThreadMembersUpdateEvent, TypingStartEvent, VoiceServerUpdateEvent,
};
use serenity::model::gateway::Presence;
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
use sled::IVec;

lazy_static! {
    static ref USERDB: sled::Db = sled::open("user_db").unwrap();
    static ref SHARED_KEY: Vec<u8> = {
        let key = std::env::var("SHARED_KEY").expect("SHARED_KEY env variable missing");
        base64::decode_config(key, base64::URL_SAFE_NO_PAD)
            .expect("Failed to decode base64 SHARED_KEY")
    };
}

struct Handler;

/// Modifies the name of the user to either sanitize it or assign it the ✓
async fn modify_name(ctx: &Context, mem: &mut Member) {
    mem.edit(&ctx.http, |m| m.nickname("Testing"))
        .await
        .expect("Failed to set display name of user");
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        // create UTexas Verified role
        if let Ok(role) = guild
            .create_role(&ctx.http, |r| r.hoist(true).name("UTexas Verified"))
            .await
        {
            println!("Created role {}", role.name);
        }
        // assign roles and modify display names
        for (uid, mut mem) in guild.members {
            println!("{:?}: {}", uid, mem.display_name());
            modify_name(&ctx, &mut mem).await;
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        modify_name(&ctx, &mut new_member).await;
    }

    async fn guild_member_update(&self, ctx: Context, update: GuildMemberUpdateEvent) {
        if let Some(new_nick) = update.nick {
            // using special characters so set it to their uname
            if let Ok(guild) = ctx.http.get_guild(update.guild_id.into()).await {
                if let Ok(mut member) = guild.member(&ctx.http, update.user.id).await {
                    modify_name(&ctx, &mut member).await;
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
                    Some(guild) => handlers::scan(guild, ctx).await,
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
