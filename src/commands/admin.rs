use std::str::FromStr;

use poise::{send_reply, serenity_prelude::Channel};
use time::Duration;

use crate::{
    commands::{arguments::*, camel_slug::slugify_camel, *},
    jam_types::JamType,
};

#[poise::command(slash_command, guild_only, subcommands("exchange_create"))]
pub async fn exchange(_ctx: Context<'_>) -> CommandResult {
    Err(anyhow!("/reb exchange command should never be executed").into())
}

/// Create a rating exchange.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, rename = "create")]
pub async fn exchange_create(
    ctx: ApplicationContext<'_>,

    #[rename = "type"]
    #[description = "The jam type."]
    jam_type: JamType,

    #[rename = "link"]
    #[description = "The jam link. Must correspond to the jam type."]
    jam_link: TrimmedString,

    #[description = "The display name of the exchange to use in announcements."]
    display_name: TrimmedString,

    #[description = "The channel to post round announcements in."] submissions_channel: Channel,

    #[description = "The number of rounds. Defaults to 5."]
    #[min = 1]
    #[max = 32]
    rounds: Option<u8>,

    #[description = "The number of games assigned to each member. Defaults to 5."]
    #[min = 1]
    #[max = 32]
    games_per_member: Option<u8>,

    #[description = "The date and time of the first round. Defaults to now."] _start: Option<
        HumanDateTime,
    >,

    #[description = "The duration of submission period. May be shorter than the round duration. Defaults to 24 hours."]
    submission_duration: Option<HumanDuration>,

    #[description = "The duration of a round. Defaults to submission duration."]
    round_duration: Option<HumanDuration>,

    #[description = "The name of the exchange to use in commands. Must consist only of `A-Za-z0-9_-`."]
    slug: Option<ExchangeSlug>,
) -> CommandResult {
    // To validate the jam link, we need to know the jam type. So, we do it here.
    let _jam_link = jam_type
        .normalize_jam_link(jam_link.as_ref())
        .ok_or(invalid_argument(format!(
            "Invalid jam link: `{jam_link}`.\nFor {jam_type}, it should look like this: `{}`",
            jam_type.jam_link_example()
        )))?;

    let jam_slug = slug.unwrap_or_else(|| slugify_camel(display_name.as_ref()).into());
    if ExchangeSlug::from_str(jam_slug.as_ref()).is_err() {
        Err(anyhow!(
            "Auto-generated exchange slug is invalid: `{jam_slug}`."
        ))?;
    }

    // TODO: Implement custom GuildChannel argument type.
    let _submissions_channel = match submissions_channel {
        Channel::Guild(channel) => channel,
        Channel::Category(_) => {
            send_reply(ctx.into(), |reply| {
                reply.ephemeral(true).content(
                    "Invalid `submissions_channel` value: categories are not supported, please provide a text channel."
                )
            })
            .await?;
            return Ok(());
        }
        channel => {
            send_reply(ctx.into(), |reply| {
                reply.ephemeral(true).content(format!(
                    "Something went wrong, submissions channel should never be `{channel:?}`."
                ))
            })
            .await?;
            return Ok(());
        }
    };

    let _rounds = rounds.unwrap_or(5);
    let _games_per_member = games_per_member.unwrap_or(5);

    let submission_duration = submission_duration
        .map(|d| d.into())
        .unwrap_or(Duration::hours(24));
    let _round_duration = round_duration
        .map(|d| d.into())
        .unwrap_or(submission_duration);

    ctx.say("Not implemented yet").await?;

    Ok(())
}
