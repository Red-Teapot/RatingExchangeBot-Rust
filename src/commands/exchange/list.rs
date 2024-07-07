use poise::CreateReply;
use time::OffsetDateTime;

use crate::{
    commands::{internal_err, ApplicationContext, CommandResult},
    utils::formatting::format_utc,
};

#[poise::command(slash_command, rename = "list")]
pub async fn list(ctx: ApplicationContext<'_>) -> CommandResult {
    let upcoming_exchanges = ctx
        .data
        .exchange_repository
        .get_upcoming_exchanges_in_guild(
            ctx.guild_id().ok_or(internal_err(
                "This command should be executed only in a guild",
            ))?,
            OffsetDateTime::now_utc().into(),
        )
        .await;

    match upcoming_exchanges {
        Ok(exchanges) if exchanges.is_empty() => {
            ctx.send(
                CreateReply::default()
                    .content("# There are no upcoming exchanges")
                    .ephemeral(true),
            )
            .await?;
        }

        Ok(exchanges) => {
            let list = exchanges.iter().fold(String::new(), |acc, exchange| {
                acc + &format!(
                    " - **{}** (slug: `{}`) - runs from {} UTC to {} UTC\n",
                    exchange.display_name,
                    exchange.slug,
                    format_utc(exchange.submissions_start),
                    format_utc(exchange.submissions_end),
                )
            });

            ctx.send(
                CreateReply::default()
                    .content(&format!("# Upcoming exchanges:\n{list}"))
                    .ephemeral(true),
            )
            .await?;
        }

        Err(err) => {
            return Err(internal_err(&format!(
                "Could not get the upcoming exchanges: {err}"
            )));
        }
    }

    Ok(())
}
