mod db;
mod handlers;

use std::collections::{HashMap, HashSet};
use std::env;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use aws_sdk_sqs::model::DeleteMessageBatchRequestEntry;
use serde::Deserialize;
use serenity::http::GuildPagination;
use serenity::model::guild::{Guild, Member, PartialGuild, Role};
use serenity::model::id::{GuildId, RoleId, UserId};
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

const REQUESTS_PER_SECOND: i32 = 10;
const SQS_BECOME_VERIFIED_REQUEST_URL: &'static str = "https://sqs.us-east-1.amazonaws.com/402762806873/on-verification-update";

type IgnoreSet = Arc<tokio::sync::Mutex<HashSet<UserId>>>;

struct Handler {
    db_client: &'static db::DynamoDB,
    // used to ignore an additional invocation of GuildMemberUpdateEvent
    ignore_set: IgnoreSet,
    background_task_running: AtomicBool,
}

/// Scans all users in the guild to check nickname compliance
async fn rescan(
    user_db: &'static db::DynamoDB,
    command: ApplicationCommandInteraction,
    guild: GuildId,
    ctx: Context,
    ignore_set: IgnoreSet,
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
        output = match scan(user_db, guild, ctx.clone(), ignore_set.clone()).await {
            Ok(n) => format!(
                "Command Sent Successfully (should complete in {} seconds)",
                n
            ),
            Err(e) => format!("Command Failed: {}", e.to_string()),
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
    user_db: &'static db::DynamoDB,
    guild_id: GuildId,
    ctx: Context,
    ignore_set: IgnoreSet,
) -> serenity::Result<String> {
    let guild = ctx.http.get_guild(guild_id.into()).await?;
    let mut guild_members = guild.members(&ctx.http, None, None).await?;
    let numbers = match guild_members.len() {
        1000 => "≥250".to_string(),
        _ => format!("~{}", guild_members.len() / 10),
    };
    tokio::spawn(async move {
        let role_mappings = user_db.get_role_config(guild_id).await;
        // for pagination
        while guild_members.len() > 0 {
            let mut last_id = None;
            for mut member in &mut guild_members {
                handle_member_status(
                    user_db,
                    &ctx,
                    &mut member,
                    &role_mappings,
                    ignore_set.clone(),
                )
                .await;
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
async fn handle_member_status(
    db_client: &db::DynamoDB,
    ctx: &Context,
    mem: &mut Member,
    role_mappings: &HashMap<String, u64>,
    ignore_set: IgnoreSet,
) -> bool {
    let original = mem.display_name().to_string();
    let mut cleaned = mem
        .display_name()
        .replace(|c: char| !c.is_ascii(), "")
        .trim()
        .to_string();
    if let Some(user_claims) = db_client.get_user(mem.user.id.into()).await {
        let mut roles_to_add = Vec::new();
        let mut user_tags = user_claims.affiliation.clone();
        user_tags.extend(user_claims.major.clone());
        user_tags.extend(user_claims.school.clone());
        for tag in &user_tags {
            if let Some(role_id) = role_mappings.get(tag) {
                if !mem.roles.contains(&RoleId(*role_id)) {
                    roles_to_add.push(RoleId(*role_id));
                }
            }
        }
        if roles_to_add.len() > 0 && !mem.add_roles(&ctx.http, &roles_to_add).await.is_ok() {
            eprintln!("Failed to Add Roles to {}", original);
        }
        if user_claims.affiliation.contains(&"student".to_string()) {
            cleaned.push_str(" ✓");
        } else {
            return true;
        }
    }
    if original != cleaned {
        {
            ignore_set.lock().await.insert(mem.user.id);
        }
        mem.edit(&ctx.http, |m| m.nickname(cleaned)).await.is_ok()
    } else {
        false
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, ctx: Context, guild: Guild) {
        scan(self.db_client, guild.id, ctx, self.ignore_set.clone())
            .await
            .unwrap();
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        let role_mappings = self.db_client.get_role_config(guild_id).await;
        handle_member_status(
            self.db_client,
            &ctx,
            &mut new_member,
            &role_mappings,
            self.ignore_set.clone(),
        )
        .await;
    }

    async fn guild_member_update(&self, ctx: Context, update: GuildMemberUpdateEvent) {
        {
            let mut ignore_set = self.ignore_set.lock().await;
            if ignore_set.contains(&update.user.id) {
                ignore_set.remove(&update.user.id);
                return;
            }
        }
        if let Ok(guild) = ctx.http.get_guild(update.guild_id.into()).await {
            let role_mappings = self.db_client.get_role_config(guild.id).await;
            if let Ok(mut member) = guild.member(&ctx.http, update.user.id).await {
                handle_member_status(
                    self.db_client,
                    &ctx,
                    &mut member,
                    &role_mappings,
                    self.ignore_set.clone(),
                )
                .await;
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
                    command.name("rescan").description(
                        "Check all users in the guild for nickname compliance and role assignment",
                    )
                })
        })
        .await;

        println!(
            "I now have the following global slash commands: {:#?}",
            commands
        );

        let ctx = Arc::new(ctx);

        if !self
            .background_task_running
            .fetch_or(true, Ordering::Relaxed)
        {
            let ctx1 = ctx.clone();

            let dbc = self.db_client;
            let igset = self.ignore_set.clone();
            tokio::spawn(async move {
                let config = aws_config::load_from_env().await;
                let client = aws_sdk_sqs::Client::new(&config);

                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    let out = client
                        .receive_message()
                        .queue_url(SQS_BECOME_VERIFIED_REQUEST_URL)
                        .max_number_of_messages(REQUESTS_PER_SECOND)
                        .send()
                        .await
                        .unwrap();

                    let messages = match out.messages {
                        Some(msgs) => msgs,
                        None => continue,
                    };

                    let mut entries = Vec::new();

                    for msg in messages {
                        let body = msg.body.expect("invalid message received");
                        let req: BecomeVerifiedMessage =
                            serde_json::from_str(&body).expect("invalid message received");
                        let discord_id: u64 = req.discord_id.parse().unwrap_or(0);
                        if let Ok(guilds) = ctx1.http.get_guilds(&GuildPagination::After(GuildId(0)), 100).await {
                            for guild in guilds {
                                if let Ok(mut member) = ctx1.http.get_member(guild.id.into(), discord_id).await {
                                    let role_mappings = dbc.get_role_config(guild.id).await;
                                    handle_member_status(dbc, &ctx1, &mut member, &role_mappings, igset.clone()).await;
                                }
                            }
                        }
                        entries.push(
                            DeleteMessageBatchRequestEntry::builder()
                                .set_id(Some(entries.len().to_string()))
                                .set_receipt_handle(msg.receipt_handle)
                                .build(),
                        );
                    }

                    client
                        .delete_message_batch()
                        .queue_url(SQS_BECOME_VERIFIED_REQUEST_URL)
                        .set_entries(Some(entries))
                        .send()
                        .await
                        .unwrap();
                }
            });
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if let Err(why) = match command.data.name.as_str() {
                "verify" => handlers::verify(command, ctx).await,
                "rescan" => match command.guild_id {
                    Some(guild) => {
                        rescan(self.db_client, command, guild, ctx, self.ignore_set.clone()).await
                    }
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
    let db_client = Box::leak(Box::new(db::DynamoDB::new("users").await));
    let ignore_set = Arc::new(Mutex::new(HashSet::new()));
    // Build our client.
    let mut client = Client::builder(token)
        .intents(GatewayIntents::GUILD_MEMBERS)
        .event_handler(Handler {
            db_client,
            ignore_set,
            background_task_running: AtomicBool::new(false),
        })
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

#[derive(Deserialize)]
struct BecomeVerifiedMessage {
    discord_id: String
}
