use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    utils::Color,
};

pub async fn utverify(
    command: ApplicationCommandInteraction,
    ctx: Context,
) -> serenity::Result<()> {
    command
        .create_interaction_response(ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.create_embed(|embed| embed.title("Success")))
        })
        .await
}

pub fn utvhelp<'a>(
    embed: &'a mut CreateEmbed,
    _command: &ApplicationCommandInteraction,
) -> &'a mut CreateEmbed {
    embed
        .title("UTexas Verify Help Page")
        .color(Color::from_rgb(0, 255, 0))
        .description("Use the `/utverify` command to verify your discord account...")
}

pub fn unknown_command<'a>(
    embed: &'a mut CreateEmbed,
    _command: &ApplicationCommandInteraction,
) -> &'a mut CreateEmbed {
    embed
        .title("Incorrect Command Usage")
        .description("Use one of 2 commands: `utverify`, `utvhelp`, and make sure your input values are valid.")
        .color(Color::from_rgb(255, 0, 0))
}
