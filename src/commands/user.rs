use poise::samples::HelpConfiguration;

use crate::commands::*;

/// Provides help for the available bot commands.
#[poise::command(slash_command, ephemeral)]
pub async fn help(
    ctx: Context<'_>,

    #[description = "The command to provide help about."]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> CommandResult {
    let config = HelpConfiguration {
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;

    Ok(())
}

/// Submits your game to the active review exchange.
#[poise::command(slash_command, ephemeral)]
pub async fn submit(
    ctx: Context<'_>,

    #[description = "Exchange name"] _exchange: String,

    #[description = "Game link"] _link: String,
) -> CommandResult {
    ctx.say("Submitting games is not implemented yet").await?;
    Ok(())
}

/// Revokes your submission from the active review exchange.
#[poise::command(slash_command, ephemeral)]
pub async fn revoke(
    ctx: Context<'_>,

    #[description = "Exchange name"] _exchange: String,
) -> CommandResult {
    ctx.say("Revoking games is not implemented yet").await?;
    Ok(())
}

/// Registers the game as played, so the bot won't assign it to you.
#[poise::command(slash_command, ephemeral)]
pub async fn played(ctx: Context<'_>, #[description = "Game link"] _link: String) -> CommandResult {
    ctx.say("Registering played games is not implemented yet")
        .await?;
    Ok(())
}
