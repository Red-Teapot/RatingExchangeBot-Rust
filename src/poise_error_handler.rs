use poise::{Context, CreateReply, FrameworkError};
use tracing::{error, warn};

use crate::{commands::CommandError, BotState};

pub async fn handle_error(error: poise::FrameworkError<'_, BotState, CommandError>) {
    use FrameworkError::*;

    match error {
        Setup { error, .. } => {
            error!("Error in user data setup: {}", error);
        }

        EventHandler { error, event, .. } => {
            error!(
                "Error in user event {} handler: {}",
                event.snake_case_name(),
                error
            );
        }

        Command { error, ctx, .. } => match error {
            CommandError::User { message } => {
                reply_with_error(ctx, &message).await;
            }

            CommandError::Internal { message } => {
                reply_with_internal_error(ctx, &message).await;
                error!("Internal error: {}", message);
            }

            CommandError::Serenity(error) => {
                reply_with_internal_error(ctx, &error.to_string()).await;
                error!("Serenity error: {}", error);
            }
        },

        ArgumentParse {
            error, input, ctx, ..
        } => {
            let usage = ctx
                .command()
                .help_text
                .as_deref()
                .unwrap_or("Please check the help menu or contact the admins.");

            let response = if let Some(input) = input {
                format!(
                    "**Sorry, cannot parse `{}` as an argument: {}**\n{}",
                    input, error, usage
                )
            } else {
                format!("**{}**\n{}", error, usage)
            };

            reply_with_error(ctx, &response).await;
        }

        CommandStructureMismatch {
            description, ctx, ..
        } => {
            error!(
                "Failed to deserialize interaction arguments for `{}`: {}",
                ctx.command.qualified_name, description
            );
        }

        CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            reply_with_error(
                ctx,
                &format!(
                    "Sorry, you're too fast. Please try again in {} s.",
                    remaining_cooldown.as_secs()
                ),
            )
            .await;
        }

        MissingBotPermissions { ctx, .. } => {
            reply_with_error(
                ctx,
                "Sorry, the bot lacks permissions necessary to execute this command.",
            )
            .await;
        }

        MissingUserPermissions { ctx, .. } => {
            reply_with_error(
                ctx,
                "Sorry, you don't have permissions necessary to run this command.",
            )
            .await;
        }

        NotAnOwner { ctx, .. } => {
            reply_with_error(ctx, "Sorry, only the server owner can run this command.").await;
        }

        GuildOnly { ctx, .. } => {
            reply_with_error(ctx, "Sorry, but you can only run this command in a server.").await;
        }

        DmOnly { ctx, .. } => {
            reply_with_error(ctx, "Sorry, but you can only run this command in bot DMs.").await;
        }

        NsfwOnly { ctx, .. } => {
            reply_with_error(
                ctx,
                "Sorry, but you can only run this command in a NSFW channel.",
            )
            .await;
        }

        CommandCheckFailed { error, ctx, .. } => {
            let message = if let Some(error) = error {
                format!(
                    "Sorry, can't run this command due to a failed command check: {}",
                    error
                )
            } else {
                "Sorry, can't run this command due to a failed command check.".to_string()
            };

            reply_with_error(ctx, &message).await;
        }

        DynamicPrefix { error, msg, .. } => {
            error!(
                "Dynamic prefix failed for message {:?}: {}",
                msg.content, error
            );
        }

        UnknownCommand {
            prefix,
            msg_content,
            ..
        } => {
            warn!(
                "Recognized prefix {:?} but didn't recognize the command name in {:?}",
                prefix, msg_content
            );
        }

        UnknownInteraction { interaction, .. } => {
            warn!("Received an unknown interaction: {:?}", interaction);
        }

        error => {
            error!("Unknown error: {}", error);
        }
    }
}

async fn reply_with_error(ctx: Context<'_, BotState, CommandError>, error_message: &str) {
    if let Err(send_error) = poise::send_reply(
        ctx,
        CreateReply::default()
            .content(error_message)
            .ephemeral(true),
    )
    .await
    {
        error!(
            "Failed to send an error message to the user: {}\nThe message was: {}",
            send_error, error_message
        );
    }
}

async fn reply_with_internal_error(ctx: Context<'_, BotState, CommandError>, error_message: &str) {
    reply_with_error(
        ctx,
        &format!(
            "Sorry, there was an internal error while executing your command: {}",
            error_message
        ),
    )
    .await;
}
