use std::str::FromStr;

use poise::{
    send_reply,
    serenity_prelude::{Channel, Mentionable},
};
use time::{Duration, OffsetDateTime};

use crate::{
    commands::{arguments::*, camel_slug::slugify_camel, *},
    jam_types::JamType,
    storage::CreateExchange,
    utils::{timestamp, TimestampStyle},
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

    #[description = "The date and time of the first round. Defaults to now."] start: Option<
        HumanDateTime,
    >,

    #[description = "The duration of submission period. Defaults to round duration."]
    submission_duration: Option<HumanDuration>,

    #[description = "The duration of a round. Defaults to 24 hours."] round_duration: Option<
        HumanDuration,
    >,

    #[description = "The name of the exchange to use in commands. Must consist only of `A-Za-z0-9_-`."]
    slug: Option<ExchangeSlug>,
) -> CommandResult {
    // To validate the jam link, we need to know the jam type. So, we do it here.
    let jam_link = jam_type
        .normalize_jam_link(jam_link.as_ref())
        .ok_or(invalid_argument(format!(
            "Invalid jam link: `{jam_link}`.\nFor {jam_type}, it should look like this: `{}`",
            jam_type.jam_link_example()
        )))?;

    let slug = slug.unwrap_or_else(|| slugify_camel(display_name.as_ref()).into());
    if ExchangeSlug::from_str(slug.as_ref()).is_err() {
        Err(anyhow!(
            "Auto-generated exchange slug is invalid: `{slug}`."
        ))?;
    }

    // TODO: Implement custom GuildChannel argument type.
    let submission_channel = match submissions_channel {
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

    let rounds = rounds.unwrap_or(5);
    let games_per_member = games_per_member.unwrap_or(5);

    let start = start
        .map(|dt| dt.materialize(OffsetDateTime::now_utc()))
        .unwrap_or(OffsetDateTime::now_utc());

    let round_duration = round_duration
        .map(|d| d.into())
        .unwrap_or(Duration::hours(24));

    let submission_duration = submission_duration
        .map(|d| d.into())
        .unwrap_or(round_duration);

    let (exchange, rounds) = ctx
        .data
        .exchange_storage
        .create_exchange(CreateExchange {
            guild_id: ctx.guild_id().unwrap(),
            jam_type,
            jam_link,
            slug: slug.to_string(),
            display_name: display_name.to_string(),
            submission_channel: submission_channel.id,
            num_rounds: rounds,
            first_round_start: start,
            submission_duration,
            round_duration,
            games_per_member,
        })
        .await
        .unwrap();

    ctx.send(|reply| {
        reply.embed(|embed| {
            let rounds_str = rounds
                .iter()
                .map(|round| {
                    let start_ts = timestamp(
                        round.submissions_start_at.into(),
                        TimestampStyle::ShortDateTime,
                    );
                    let end_ts = timestamp(
                        round.assignments_sent_at.into(),
                        TimestampStyle::ShortDateTime,
                    );

                    format!("{start_ts} - {end_ts}\n")
                })
                .collect::<String>();
            embed
                .title("Exchange created!")
                .color(0x00FF00)
                .field("Name", exchange.display_name, true)
                .field("Slug", &format!("`{}`", exchange.slug), true)
                .field(
                    "Submission channel",
                    exchange.submission_channel.0.mention(),
                    true,
                )
                .field("Jam type", exchange.jam_type.0.name(), true)
                .field("Jam link", exchange.jam_link, true)
                .field("Rounds", rounds_str.trim(), false)
        })
    })
    .await?;

    Ok(())
}
