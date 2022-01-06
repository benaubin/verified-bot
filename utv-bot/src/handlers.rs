use serenity::model::prelude::{Guild, GuildId, Message};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    utils::Color,
};

pub async fn verify(command: ApplicationCommandInteraction, ctx: Context) -> serenity::Result<()> {
    command
        .create_interaction_response(ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| msg.create_embed(|embed| embed.title("Success")))
        })
        .await
}

/// Scans all users in the guild to check nickname compliance
pub async fn scan(guild: GuildId, ctx: Context) -> serenity::Result<()> {
    let guild = ctx.http.get_guild(guild.into()).await?;
    todo!()
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
