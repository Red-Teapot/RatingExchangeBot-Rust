use indoc::formatdoc;
use poise::CreateReply;
use time::OffsetDateTime;
use tracing::debug;

use crate::{
    commands::{internal_err, user_err, ApplicationContext, CommandResult},
    models::{types::UtcDateTime, NewSubmission},
    utils::formatting::{format_local, format_utc},
};

#[poise::command(slash_command, rename = "submit")]
pub async fn submit(
    ctx: ApplicationContext<'_>,
    #[description = "Submission link"] link: String,
) -> CommandResult {
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
                        **There are no currently active exchanges in this channel.**

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

    let submission = NewSubmission {
        exchange_id: exchange.id,
        link,
        submitter: ctx.author().id,
        submitted_at: UtcDateTime::from(OffsetDateTime::now_utc()),
    };

    let mut message: String = formatdoc! {
        r#"
            **Submitted!**

            You will receive your assignments in the DMs when the exchange ends: {end_local} your time or {end_utc} UTC.
        "#,
        end_local = format_local(exchange.submissions_end),
        end_utc = format_utc(exchange.submissions_end),
    };

    if let Ok(Some(conflict)) = ctx
        .data
        .submission_repository
        .get_conflicting_submission(&submission)
        .await
    {
        if submission.link == conflict.link {
            let message = formatdoc! {
                r#"
                    **Someone else has already submitted this link**

                    If you worked in a team, only one team member can submit an entry and get assignments.
                "#,
            };
            return Err(user_err(message));
        }

        if submission.submitter == conflict.submitter {
            message = formatdoc! {
                r#"
                    **Updated your submission**

                    Previously submitted link: `{old_link}`.

                    New link: `{new_link}`.

                    You will receive your assignments in the DMs when the exchange ends: {end_local} your time or {end_utc} UTC.
                "#,
                old_link = conflict.link,
                new_link = submission.link,
                end_local = format_local(exchange.submissions_end),
                end_utc = format_utc(exchange.submissions_end),
            };
        }
    }

    match ctx
        .data
        .submission_repository
        .add_or_update_submission(&submission)
        .await
    {
        Ok(_) => {
            ctx.send(CreateReply::default().ephemeral(true).content(message))
                .await?;
            Ok(())
        }
        Err(err) => Err(internal_err(format!(
            "Could not add/update submission: {err}"
        ))),
    }
}
