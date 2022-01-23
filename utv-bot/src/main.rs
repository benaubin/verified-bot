mod handlers;
mod db;

use std::env;

use lazy_static::lazy_static;
use serenity::model::guild::{
    Guild, Member, PartialGuild, Role
};
use serenity::model::id::{
    GuildId, RoleId,
};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
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

lazy_static! {
    static ref ROLEDB: sled::Db = sled::open("role_db").unwrap();
    static ref SHARED_KEY: Vec<u8> = {
        let key = std::env::var("SHARED_KEY").expect("SHARED_KEY env variable missing");
        base64::decode_config(key, base64::URL_SAFE_NO_PAD)
            .expect("Failed to decode base64 SHARED_KEY")
    };
}

struct Handler {
    db_client: &'static db::UserDB,

}

/// Scans all users in the guild to check nickname compliance
async fn rescan(
    user_db: &'static db::UserDB,
    command: ApplicationCommandInteraction,
    guild: GuildId,
    ctx: Context,
) -> serenity::Result<()> {
    let output;
    if !command
        .member
        .as_ref()
        .unwrap()
        .permissions
        .unwrap()
        .administrator()
    {
        output = "You must be an administrator to run this command.".to_string();
    } else {
        output = match scan(user_db, guild, ctx.clone()).await {
            Ok(n) => format!("Command Sent Successfully (should complete in {} seconds)", n),
            Err(e) => format!("Command Failed: {}", e.to_string())
        };
    }
    command
        .create_interaction_response(&ctx.http, |interaction| {
            interaction
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|embed| embed.title(output))
                })
        })
        .await
}

async fn scan(
    user_db: &'static db::UserDB,
    guild_id: GuildId,
    ctx: Context,
) -> serenity::Result<String> {
    let guild = ctx.http.get_guild(guild_id.into()).await?;
    let mut guild_members = guild.members(&ctx.http, None, None).await?;
    let numbers = match guild_members.len() {
        1000 => "≥250".to_string(),
        _ => format!("~{}", guild_members.len() / 10)
    };
    tokio::spawn(async move {
        // for pagination
        while guild_members.len() > 0 {
            let mut last_id = None;
            for mut member in &mut guild_members {
                handle_member_status(user_db, &ctx, &mut member).await;
                last_id = Some(member.user.id);
                // sleep to stay far away from rate limit
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            guild_members = guild.members(&ctx.http, None, last_id).await.unwrap();
        }
    });
    Ok(numbers)
}

/// Modifies the name and roles of the user to either sanitize it or assign it the ✓
async fn handle_member_status(user_db: &db::UserDB, ctx: &Context, mem: &mut Member) -> bool {
    let guild = ctx.http.get_guild(mem.guild_id.into()).await.unwrap();
    let verified_role = get_verified_role(&ctx, &guild).await;
    let original = mem.display_name().to_string();
    let mut cleaned = mem.display_name().replace("✓", "_");
    if user_db.user_exists(mem.user.id.into()).await {
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
        mem.edit(&ctx.http, |m| m.nickname(cleaned)).await.is_ok()
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
            let role_id = RoleId(u64::from_be_bytes(
                value.to_vec().as_slice().try_into().unwrap(),
            ));
            guild.roles.get(&role_id).unwrap().id
        }
        None => {
            let new_role = guild
                .create_role(&ctx.http, |r| {
                    r.name("UTexas Verified")
                        .hoist(true)
                        .mentionable(true)
                        .colour(0xbf5700)
                })
                .await
                .unwrap();
            ROLEDB
                .insert(key, new_role.id.as_u64().to_be_bytes().to_vec())
                .unwrap();
            new_role.id
        }
    };
    let role = guild.roles.get(&role_id).unwrap();
    role
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        scan(self.db_client, guild.id, ctx).await.unwrap();
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        handle_member_status(self.db_client, &ctx, &mut new_member).await;
    }

    async fn guild_member_update(&self, ctx: Context, update: GuildMemberUpdateEvent) {
        if let Some(new_nick) = update.nick {
            if let Ok(guild) = ctx.http.get_guild(update.guild_id.into()).await {
                if let Ok(mut member) = guild.member(&ctx.http, update.user.id).await {
                    handle_member_status(self.db_client, &ctx, &mut member).await;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
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
                        .description("Check all users in the guild for nickname compliance and role assignment")
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
                    Some(guild) => rescan(self.db_client, command, guild, ctx).await,
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

    // DynamoDB Client
    let db_client = Box::leak(Box::new(db::UserDB::new("users").await));
    // Build our client.
    let mut client = Client::builder(token)
        .intents(GatewayIntents::GUILD_MEMBERS)
        .event_handler(Handler { db_client })
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
