#![forbid(unsafe_code)]
#![allow(dead_code)] // TODO: Remove this before the first release.

mod actors;
mod assignment_service;
mod commands;
mod data;
mod env_vars;
mod jam_types;
mod poise_error_handler;
mod solver;
mod storage;
mod utils;

use std::sync::Arc;

use assignment_service::AssignmentService;

use poise::serenity_prelude::{self as serenity, GuildId};
use poise_error_handler::handle_error;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use storage::ExchangeStorage;
use tracing::{error, info, info_span, Instrument};
use tracing_subscriber::prelude::*;

#[derive(Debug)]
pub struct BotState {
    pub exchange_storage: Arc<ExchangeStorage>,
}

#[tracing::instrument]
#[tokio::main]
async fn main() {
    if let Err(err) = envmnt::load_file(".env") {
        eprintln!("Could not load .env file: {err}");
    }

    if !env_vars::check() {
        eprintln!("Failed to check environment variable values");
        std::process::exit(255);
    }

    let _sentry_guard = if let Some(url) = env_vars::SENTRY_URL.option() {
        let guard = Some(sentry::init((
            url.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 1.0,
                ..Default::default()
            },
        )));

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(sentry_tracing::layer())
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        guard
    } else {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        None
    };

    let bot_state = {
        let pool = match setup_database().await {
            Ok(pool) => pool,
            Err(err) => {
                error!("Could not setup database: {err}");
                std::process::exit(255);
            }
        };

        BotState {
            exchange_storage: Arc::new(ExchangeStorage::new(pool)),
        }
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::reb()],
            on_error: |error| {
                Box::pin(async move {
                    handle_error(error).await;
                })
            },
            ..Default::default()
        })
        .token(env_vars::DISCORD_BOT_TOKEN.required())
        .intents(serenity::GatewayIntents::GUILD_MESSAGES)
        .setup(|ctx, _ready, framework| {
            Box::pin(
                async move {
                    let commands = &framework.options().commands;

                    if env_vars::REGISTER_COMMANDS_GLOBALLY.get_bool(false) {
                        info!("Registering commands globally");
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                    }

                    if let Some(guilds_str) = env_vars::REGISTER_COMMANDS_IN_GUILDS.option() {
                        let guilds = guilds_str
                            .split(',')
                            .map(|s| s.trim())
                            .map(|s| s.parse::<u64>().unwrap())
                            .map(GuildId);

                        for guild in guilds {
                            let guild_name = ctx
                                .http
                                .get_guild(guild.0)
                                .await
                                .map(|g| g.name)
                                .unwrap_or("???".to_string());

                            info!("Registering commands in guild {guild} ({guild_name})");

                            poise::builtins::register_in_guild(ctx, commands, guild).await?;
                        }
                    }

                    AssignmentService::create_and_start(bot_state.exchange_storage.clone());

                    Ok(bot_state)
                }
                .instrument(info_span!("bot_setup")),
            )
        });

    framework.run().await.unwrap();
}

async fn setup_database() -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .connect(&env_vars::DATABASE_URL.required())
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
