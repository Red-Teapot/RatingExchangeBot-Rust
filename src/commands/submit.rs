use indoc::formatdoc;
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    commands::{internal_err, user_err, ApplicationContext, CommandResult},
    models::{types::UtcDateTime, NewSubmission},
};

#[poise::command(slash_command, rename = "submit")]
pub async fn submit(
    ctx: ApplicationContext<'_>,
    #[description = "Exchange slug"] slug: String,
    #[description = "Submission link"] link: String,
) -> CommandResult {
    let exchange = {
        let guild_id = ctx.guild_id().ok_or(internal_err("Guild id not found"))?;
        let channel_id = ctx.channel_id();
        let now = UtcDateTime::from(OffsetDateTime::now_utc());

        match ctx
            .data
            .exchange_repository
            .get_overlapping_exchanges(guild_id, channel_id, &slug, now, now)
            .await
        {
            Ok(exchanges) if exchanges.is_empty() => {
                let message = formatdoc! {
                    r#"
                        **There are no active exchanges with slug `%{slug}` in this channel.**

                        Check the starting and ending dates of the exchanges and their submission channels.
                    "#,
                    slug = slug,
                };
                return Err(user_err(message));
            }

            Ok(mut exchanges) if exchanges.len() == 1 => {
                exchanges.pop().expect("Length is checked by guard")
            }

            Ok(exchanges) => {
                let exchange_ids = exchanges
                    .iter()
                    .map(|e| format!("{:?}", e.id))
                    .collect::<Vec<String>>()
                    .join(", ");
                return Err(internal_err(&format!("Too many exchanges: {exchange_ids}")));
            }

            Err(err) => {
                return Err(internal_err(&format!("Could not get exchanges: {err}")));
            }
        }
    };

    debug!(
        "Found matching exchange: {} (id {:?})",
        &exchange.slug, exchange.id
    );

    let link = {
        let jam_type = exchange.jam_type;
        let jam_link = exchange.jam_link;

        match jam_type.normalize_jam_entry_link(&jam_link, &link) {
            Some(link) => link,
            None => {
                let message = formatdoc! {
                    r#"
                        **Your entry link is invalid.**

                        It should look like this: `{example}`.

                        Make sure to use the correct submission page.
                    "#,
                    example = jam_type.jam_entry_link_example(&jam_link),
                };
                return Err(user_err(message));
            }
        }
    };

    debug!("Normalized link: {link}");

    let submission = NewSubmission {
        exchange_id: exchange.id,
        link: link,
        submitter: ctx.author().id,
        submitted_at: UtcDateTime::from(OffsetDateTime::now_utc()),
    };

    debug!("New submission: {submission:?}");

    match ctx
        .data
        .submission_repository
        .add_or_update_submission(&submission)
        .await
    {
        Ok(submission) => {
            debug!("??? {submission:?}");
        }
        Err(err) => {
            return Err(internal_err(format!(
                "Could not add/update submission: {err}"
            )));
        }
    };

    return Err(internal_err("Not implemented yet"));
}
