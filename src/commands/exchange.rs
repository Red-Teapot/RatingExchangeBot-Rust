use std::{str::FromStr};


use poise::serenity_prelude::{ButtonStyle, Channel, Mentionable};
use serenity::{builder::CreateEmbed, utils::Color};
use time::{format_description, macros::format_description, Duration, OffsetDateTime};

use crate::{
    commands::{arguments::*, camel_slug::slugify_camel, *},
    jam_types::JamType,
    repository::CreateExchange,
    utils::{timestamp, TimestampStyle},
};

const DATETIME_FORMAT: &[format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]");

#[poise::command(
    slash_command,
    guild_only,
    subcommands("exchange_create", "exchange_list", "exchange_delete"),
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn exchange(_ctx: Context<'_>) -> CommandResult {
    Err(user_err("The `/exchange` command is not supported yet"))
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
        .ok_or(user_err(&format!(
            "Invalid jam link: `{jam_link}`.\nFor {jam_type}, it should look like this: `{}`",
            jam_type.jam_link_example()
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
        Channel::Category(_) => {
            return Err(user_err(&format!(
                "Invalid `submissions_channel` value: categories are not supported, please provide a text channel."
            )));
        }
        channel => {
            return Err(internal_err(&format!(
                "Something went wrong, submissions channel should never be `{channel:?}`."
            )));
        }
    };

    let games_per_member = games_per_member.unwrap_or(5);

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
            .exchange_storage
            .get_overlapping_exchanges(guild, submission_channel.id, start.into(), end.into())
            .await
            .map_err(|err| {
                internal_err(&format!("Could not check for overlapping exchanges: {err}"))
            })?;

        if !overlapping_exchanges.is_empty() {
            ctx.send(|reply| {
                let mut content = concat!(
                    "# There are overlapping exchanges\n",
                    "The exchange can't be created because the following exchanges use the same submission channel and ",
                    "have overlapping submission periods:\n",
                ).to_string();

                for exchange in &overlapping_exchanges {
                    content += &format!(
                        " - **{}** (slug: `{}`) - runs from {} UTC to {} UTC\n", 
                        exchange.display_name,
                        exchange.slug,
                        OffsetDateTime::from(exchange.submissions_start).format(DATETIME_FORMAT)
                            .expect("Format should be correct since it's hardcoded"),
                        OffsetDateTime::from(exchange.submissions_end).format(DATETIME_FORMAT)
                            .expect("Format should be correct since it's hardcoded"),
                    );
                }

                reply
                    .content(content)
                    .ephemeral(true)
            })
            .await?;

            return Ok(());
        }
    }

    let confirm_timeout = Duration::minutes(5);

    let create_embed = |embed: &mut CreateEmbed, color: Color| {
        embed
            .title(&display_name)
            .color(color)
            .field("Jam type", jam_type.name(), true)
            .field("Jam link", &jam_link, true)
            .field("Submission channel", submission_channel.mention(), false)
            .field(
                "Start",
                &format!(
                    "{} UTC or {} your time",
                    start
                        .format(DATETIME_FORMAT)
                        .expect("Format should be correct since it's hardcoded"),
                    timestamp(start, TimestampStyle::ShortDateTime)
                ),
                false,
            )
            .field(
                "End",
                &format!(
                    "{} UTC or {} your time",
                    end.format(DATETIME_FORMAT)
                        .expect("Format should be correct since it's hardcoded"),
                    timestamp(end, TimestampStyle::ShortDateTime)
                ),
                false,
            )
            .field("Duration", &format!("{duration}"), false)
            .field("Games per member", &format!("{}", games_per_member), true)
            .field("Slug", &format!("`{}`", &slug), true);
    };

    let reply = ctx
        .send(|reply| {
            reply
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
                .embed(|embed| {
                    create_embed(embed, Color::GOLD);
                    embed
                })
                .components(|components| {
                    components.create_action_row(|row| {
                        row.create_button(|button| {
                            button
                                .label("Cancel")
                                .style(ButtonStyle::Secondary)
                                .custom_id("cancel")
                        })
                        .create_button(|button| {
                            button
                                .label("Confirm")
                                .style(ButtonStyle::Primary)
                                .custom_id("confirm")
                        })
                    })
                })
        })
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
                    .edit(ctx.into(), |b| {
                        b.content("# Canceled!").components(|b| b).embed(|embed| {
                            create_embed(embed, Color::RED);
                            embed
                        })
                    })
                    .await?;
            }

            "confirm" => {
                let creation_result = ctx
                    .data
                    .exchange_storage
                    .create_exchange(CreateExchange {
                        guild,
                        channel: submission_channel.id,
                        jam_type,
                        jam_link: jam_link.to_string(),
                        slug: slug.to_string(),
                        display_name: display_name.to_string(),
                        start: start.into(),
                        duration,
                        games_per_member,
                    })
                    .await;

                match creation_result {
                    Ok(_exchange) => {
                        reply
                            .edit(ctx.into(), |b| {
                                b.content("# Exchange created!")
                                    .components(|b| b)
                                    .embed(|embed| {
                                        create_embed(embed, Color::DARK_GREEN);
                                        embed
                                    })
                            })
                            .await?;
                    }
                    Err(err) => {
                        reply
                            .edit(ctx.into(), |b| {
                                b.content(&format!("# Could not create exchange!\n{err}"))
                                    .components(|b| b)
                            })
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

#[poise::command(slash_command, rename = "delete")]
pub async fn exchange_delete(
    ctx: ApplicationContext<'_>,
    #[description = "Exchange slug"] slug: String,
) -> CommandResult {
    let deletion_result = ctx
        .data
        .exchange_storage
        .delete_exchange(
            ctx.guild_id().ok_or(internal_err(
                "This command should be executed only in a guild",
            ))?,
            &slug,
        )
        .await;

    match deletion_result {
        Ok(true) => {
            ctx.send(|reply| reply.content(&format!("# Exchange `{slug}` deleted")))
                .await?;
        }

        Ok(false) => {
            return Err(user_err(&format!(
                "Exchange with slug `{slug}` does not exist"
            )));
        }

        Err(err) => {
            return Err(internal_err(&format!(
                "Could not delete the exchage: {err}"
            )));
        }
    }

    Ok(())
}

#[poise::command(slash_command, rename = "list")]
pub async fn exchange_list(ctx: ApplicationContext<'_>) -> CommandResult {
    let upcoming_exchanges = ctx
        .data
        .exchange_storage
        .get_upcoming_exchanges(
            ctx.guild_id().ok_or(internal_err(
                "This command should be executed only in a guild",
            ))?,
            OffsetDateTime::now_utc().into(),
        )
        .await;

    match upcoming_exchanges {
        Ok(exchanges) if exchanges.is_empty() => {
            ctx.send(|reply| {
                reply
                    .content("# There are no upcoming exchanges")
                    .ephemeral(true)
            })
            .await?;
        }

        Ok(exchanges) => {
            ctx.send(|reply| {
                let list = exchanges.iter().fold(String::new(), |acc, exchange| {
                    acc + &format!(
                        " - **{}** (slug: `{}`) - runs from {} UTC to {} UTC\n",
                        exchange.display_name,
                        exchange.slug,
                        OffsetDateTime::from(exchange.submissions_start)
                            .format(DATETIME_FORMAT)
                            .expect("Format should be correct since it's hardcoded"),
                        OffsetDateTime::from(exchange.submissions_end)
                            .format(DATETIME_FORMAT)
                            .expect("Format should be correct since it's hardcoded"),
                    )
                });

                reply
                    .content(&format!("# Upcoming exchanges:\n{list}"))
                    .ephemeral(true)
            })
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
