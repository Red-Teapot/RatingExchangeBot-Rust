use std::num::NonZeroU8;
use std::str::FromStr;

use poise::serenity_prelude::Mentionable;
use poise::serenity_prelude::{ButtonStyle, Channel};
use poise::{ChoiceParameter, CreateReply};
use serenity::all::{Color, CreateActionRow, CreateButton};
use serenity::builder::CreateEmbed;
use time::Duration;
use time::OffsetDateTime;

use crate::models::{ExchangeState, NewExchange};
use crate::utils::formatting::{format_local, format_utc};
use crate::{
    commands::{
        arguments::{ExchangeSlug, HumanDateTime, HumanDuration, TrimmedString},
        camel_slug::slugify_camel,
        internal_err, user_err, CommandResult,
    },
    jam_types::JamType,
};

use super::super::ApplicationContext;

/// Create a rating exchange.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, rename = "create")]
pub async fn create(
    ctx: ApplicationContext<'_>,

    #[rename = "type"]
    #[description = "The jam type."]
    jam_type: JamType,

    #[rename = "link"]
    #[description = "The jam link. Must correspond to the jam type."]
    jam_link: TrimmedString,

    #[description = "The display name of the exchange to use in announcements."]
    display_name: TrimmedString,

    #[description = "The channel to post exhange announcements and accept submissions in."]
    channel: Channel,

    #[description = "The number of games assigned to each member. Defaults to 5."]
    #[min = 1]
    #[max = 32]
    games_per_member: Option<u8>,

    #[description = "When the exchange starts. Defaults to now."] start: Option<HumanDateTime>,

    #[description = "Duration of the exchange. Defaults to 24 hours."] duration: Option<
        HumanDuration,
    >,

    #[description = "The name of the exchange to use in commands. Must consist only of `A-Za-z0-9_-`."]
    slug: Option<ExchangeSlug>,
) -> CommandResult {
    // To validate the jam link, we need to know the jam type. So, we do it here.
    let jam_link = jam_type
        .normalize_jam_link(jam_link.as_ref())
        .ok_or(user_err(format!(
            "Invalid jam link: `{link}`.\nFor {type}, it should look like this: `{link_example}`",
            link = jam_link,
            type = jam_type.name(),
            link_example = jam_type.jam_link_example()
        )))?;

    let slug = slug.unwrap_or_else(|| slugify_camel(display_name.as_ref()).into());
    if ExchangeSlug::from_str(slug.as_ref()).is_err() {
        Err(internal_err(&format!(
            "Auto-generated exchange slug is invalid: `{slug}`."
        )))?;
    }

    // TODO: Implement custom GuildChannel argument type.
    let submission_channel = match channel {
        Channel::Guild(channel) => channel,
        channel => {
            return Err(internal_err(&format!(
                "Something went wrong, submissions channel should never be `{channel:?}`."
            )));
        }
    };

    let games_per_member = NonZeroU8::new(games_per_member.unwrap_or(5))
        .ok_or(internal_err("Games per member failed to validate"))?;

    let start = start
        .map(|dt| dt.materialize(OffsetDateTime::now_utc()))
        .unwrap_or(OffsetDateTime::now_utc());

    let duration = duration.map(|d| d.into()).unwrap_or(Duration::hours(24));

    let end = start + duration;

    let guild = ctx.guild_id().ok_or(internal_err(
        "Exchange create command should only be invoked in guilds",
    ))?;

    {
        let overlapping_exchanges = ctx
            .data
            .exchange_repository
            .get_overlapping_exchanges(
                guild,
                submission_channel.id,
                slug.as_ref(),
                start.into(),
                end.into(),
            )
            .await
            .map_err(|err| {
                internal_err(&format!("Could not check for overlapping exchanges: {err}"))
            })?;

        if !overlapping_exchanges.is_empty() {
            let mut content = concat!(
                "# There are overlapping exchanges\n",
                "The exchange can't be created because the following exchanges use the same submission channel and ",
                "have overlapping submission periods or matching slug:\n",
            ).to_string();

            for exchange in &overlapping_exchanges {
                content += &format!(
                    " - **{}** (slug: `{}`) - runs from {} UTC to {} UTC\n",
                    exchange.display_name,
                    exchange.slug,
                    format_utc(exchange.submissions_start),
                    format_utc(exchange.submissions_end),
                );
            }

            ctx.send(CreateReply::default().content(content).ephemeral(true))
                .await?;

            return Ok(());
        }
    }

    let new_exchange = NewExchange {
        guild,
        channel: submission_channel.id,
        jam_type,
        jam_link: jam_link.to_string(),
        slug: slug.to_string(),
        display_name: display_name.to_string(),
        state: ExchangeState::NotStartedYet,
        submissions_start: start.into(),
        submissions_end: end.into(),
        games_per_member: games_per_member,
    };

    let confirm_timeout = Duration::minutes(5);

    let reply = ctx
        .send(CreateReply::default()
                .content(format!(
                    "# Confirm exchange creation\n\
                     You can find the details of a review exchange to be created in the embed below. \
                     If you don't see the embed, check your Discord settings.\n\
                     \n\
                     If you need to make an edit, then cancel and use the command again. \
                     You can press the up arrow key in your message box to quickly bring up the last command.
                     \n\
                     **If you don't confirm exchange creation in {confirm_timeout}, it will be cancelled automatically.**"
                ))
                .embed(create_new_exchange_embed(&new_exchange, Color::GOLD))
                .components(vec![
                    CreateActionRow::Buttons(
                        vec![
                            CreateButton::new("cancel").label("Cancel").style(ButtonStyle::Secondary),
                            CreateButton::new("confirm").label("Create").style(ButtonStyle::Primary),
                        ]
                    ),
                ])
        )
        .await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx.serenity_context())
        .author_id(ctx.author().id)
        .await;

    if let Some(interaction) = interaction {
        match interaction.data.custom_id.as_str() {
            "cancel" => {
                reply
                    .edit(
                        ctx.into(),
                        CreateReply::default()
                            .content("# Canceled!")
                            .embed(create_new_exchange_embed(&new_exchange, Color::RED)),
                    )
                    .await?;
            }

            "confirm" => {
                let creation_result = ctx
                    .data
                    .exchange_repository
                    .create_exchange(NewExchange {
                        guild,
                        channel: submission_channel.id,
                        jam_type,
                        jam_link: jam_link.to_string(),
                        slug: slug.to_string(),
                        display_name: display_name.to_string(),
                        state: ExchangeState::NotStartedYet,
                        submissions_start: start.into(),
                        submissions_end: end.into(),
                        games_per_member: games_per_member,
                    })
                    .await;

                match creation_result {
                    Ok(_exchange) => {
                        reply
                            .edit(
                                ctx.into(),
                                CreateReply::default().content("# Exchange created!").embed(
                                    create_new_exchange_embed(&new_exchange, Color::DARK_GREEN),
                                ),
                            )
                            .await?;
                    }
                    Err(err) => {
                        reply
                            .edit(
                                ctx.into(),
                                CreateReply::default()
                                    .content(format!("# Could not create exchange!\n{err}")),
                            )
                            .await?;
                    }
                }
            }

            id => {
                return Err(internal_err(&format!("Unknown interaction ID: {}", id)));
            }
        }
    }

    Ok(())
}

fn create_new_exchange_embed(exchange: &NewExchange, color: Color) -> CreateEmbed {
    let exchange_duration = OffsetDateTime::from(exchange.submissions_end)
        - OffsetDateTime::from(exchange.submissions_start);

    CreateEmbed::default()
        .title(&exchange.display_name)
        .color(color)
        .field("Jam type", exchange.jam_type.name(), true)
        .field("Jam link", &exchange.jam_link, true)
        .field(
            "Submission channel",
            exchange.channel.mention().to_string(),
            false,
        )
        .field(
            "Start",
            format!(
                "{} UTC or {} your time",
                format_utc(exchange.submissions_start),
                format_local(exchange.submissions_end),
            ),
            false,
        )
        .field(
            "End",
            format!(
                "{} UTC or {} your time",
                format_utc(exchange.submissions_end),
                format_local(exchange.submissions_end),
            ),
            false,
        )
        .field("Duration", exchange_duration.to_string(), false)
        .field(
            "Games per member",
            exchange.games_per_member.to_string(),
            true,
        )
        .field("Slug", format!("`{}`", exchange.slug), true)
}
