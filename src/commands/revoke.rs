use indoc::formatdoc;
use poise::CreateReply;
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    commands::{internal_err, user_err, ApplicationContext, CommandResult},
    models::types::UtcDateTime,
};

#[poise::command(slash_command, rename = "revoke")]
pub async fn revoke(ctx: ApplicationContext<'_>) -> CommandResult {
    let exchange = {
        let guild_id = ctx.guild_id().ok_or(internal_err("Guild id not found"))?;
        let channel_id = ctx.channel_id();
        let now = UtcDateTime::from(OffsetDateTime::now_utc());

        debug!("Guild ID: {guild_id}, channel ID: {channel_id}, now: {now:?}");

        match ctx
            .data
            .exchange_repository
            .get_running_exchange(guild_id, channel_id, now)
            .await
        {
            Ok(Some(exchange)) => exchange,

            Ok(None) => {
                let message = formatdoc! {
                    r#"
                        # There are no currently active exchanges in this channel

                        You can revoke a submission only while the corresponding exchange is running.

                        Check the starting and ending dates of the exchanges and their submission channels.
                    "#,
                };
                return Err(user_err(message));
            }

            Err(err) => {
                return Err(internal_err(&format!("Could not get exchanges: {err}")));
            }
        }
    };

    let user = ctx.author().id;

    match ctx
        .data
        .submission_repository
        .revoke(exchange.id, user)
        .await
    {
        Ok(true) => {
            let message = formatdoc! {
                r#"
                    # Revoked your submission to {exchange_name}
                "#,
                exchange_name = exchange.display_name,
            };
            ctx.send(CreateReply::default().ephemeral(true).content(message))
                .await?;
            Ok(())
        }
        Ok(false) => {
            let message = formatdoc! {
                r#"
                    # Could not find your submission to {exchange_name}

                    Either you haven't submitted to that exchange, or it has already ended.
                "#,
                exchange_name = exchange.display_name,
            };
            Err(user_err(message))
        }
        Err(err) => Err(internal_err(format!(
            "Could not revoke the submission: {err}"
        ))),
    }
}
