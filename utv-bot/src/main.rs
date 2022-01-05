mod handlers;

use std::{env, char};

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction, InteractionResponseType,
        }, event::GuildMemberUpdateEvent,
    },
    prelude::*, client::bridge::gateway::GatewayIntents,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if let Err(why) = match command.data.name.as_str() {
                "utverify" => handlers::utverify(command, ctx).await,
                _ => {
                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.create_embed(|embed| match command.data.name.as_str() {
                                        "utvhelp" => handlers::utvhelp(embed, &command),
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

    async fn guild_member_update(&self, ctx: Context, update: GuildMemberUpdateEvent) {
        if let Some(new_nick) = update.nick {
            // using special characters so set it to their uname
            if !new_nick.chars().all(|c| char::is_ascii(&c)) {
                if let Ok(member) = ctx.http.get_member(update.guild_id.into(), update.user.id.into()).await {
                    member.edit(ctx.http, |edit| {
                        edit.nickname(update.user.name)
                    }).await.unwrap();
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("utverify")
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
                        .name("utvhelp")
                        .description("Learn more about the bot and its commands")
                })
        })
        .await;

        println!(
            "I now have the following global slash commands: {:#?}",
            commands
        );
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
        .intents(GatewayIntents::GUILD_MEMBERS)
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
