use std::time::Duration;
use serenity::model::prelude::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::prelude::{
    Guild, GuildId, InteractionApplicationCommandCallbackDataFlags, Message,
};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    utils::Color,
};

pub async fn verify(command: ApplicationCommandInteraction, ctx: Context) -> serenity::Result<()> {
    let options = command
        .data
        .options
        .get(0)
        .expect("Expected EID")
        .resolved
        .as_ref()
        .expect("Expected Value");
    let mut res_ok = false;
    if let ApplicationCommandInteractionDataOptionValue::String(eid) = options {
        println!("Received EID: {}", eid);
        let client = reqwest::Client::new();
        let request_token =
            std::env::var("REQUEST_TOKEN").expect("Expected REQUEST_TOKEN variable");
        let mut eid = eid.clone();
        eid.push('\n');
        res_ok = client
            .post(request_token)
            .body(eid)
            .send()
            .await
            .is_ok();
        println!("Mail sent?: {}", res_ok);
    }
    command
        .create_interaction_response(&ctx.http, |interaction| {
            interaction.interaction_response_data(|message| {
                message
                    .create_embed(|embed| {
                        embed.title(if res_ok {
                            "Token Verification Email Sent"
                        } else {
                            "Error: Please Check You Entered Your EID Correctly"
                        })
                    })
                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
            })
        })
        .await
}

pub fn help<'a>(
    embed: &'a mut CreateEmbed,
    _command: &ApplicationCommandInteraction,
) -> &'a mut CreateEmbed {
    embed
        .title("UTexas Verify Help Page")
        .color(Color::from_rgb(0, 255, 0))
        .field(
            "`/verify`",
            "Connect your UT EID to your discord account",
            false,
        )
        .field(
            "`/rescan`",
            "**ADMIN-ONLY**: checks all users in this guild for nickname compliance",
            false,
        )
}

pub fn unknown_command<'a>(
    embed: &'a mut CreateEmbed,
    _command: &ApplicationCommandInteraction,
) -> &'a mut CreateEmbed {
    embed
        .title("Incorrect Command Usage")
        .description("Use one of 3 commands: `/verify`, `/help`, `/rescan`, and make sure your input values are valid.")
        .color(Color::from_rgb(255, 0, 0))
}
