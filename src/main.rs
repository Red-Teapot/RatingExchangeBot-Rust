#![forbid(unsafe_code)]
#![allow(dead_code)] // TODO: Remove this before the first release.

mod assignment_service;
mod commands;
mod jam_types;
mod models;
mod poise_error_handler;
mod repository;
mod solver;
mod utils;

use std::{process::exit, sync::Arc};

use assignment_service::AssignmentService;

use poise::serenity_prelude::{self as serenity, GuildId};
use poise_error_handler::handle_error;
use repository::ExchangeRepository;
use serde::Deserialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::{error, info, info_span, level_filters::LevelFilter, warn, Instrument};
use tracing_subscriber::{filter::Directive, prelude::*};

#[derive(Debug, Deserialize)]
struct AppConfig {
    discord_bot_token: String,
    database_url: String,
    register_commands_globally: Option<bool>,
    register_commands_in_guilds: Option<Vec<u64>>,
}

#[derive(Debug)]
pub struct BotState {
    pub exchange_storage: Arc<ExchangeRepository>,
}

#[tracing::instrument]
#[tokio::main]
async fn main() {
    if let Err(err) = dotenvy::dotenv() {
        warn!("Could not load config from .env file: {err}");
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    "rating_exchange_bot=info"
                        .parse()
                        .expect("Hard-coded default directive should be correct"),
                )
                .from_env_lossy(),
        )
        .init();

    let app_config = match envy::from_env::<AppConfig>() {
        Ok(config) => config,
        Err(err) => {
            error!("Could not load app config: {err}");
            exit(255);
        }
    };

    let app_state = {
        let pool = match setup_database(&app_config.database_url).await {
            Ok(pool) => pool,
            Err(err) => {
                error!("Could not setup database: {err}");
                std::process::exit(255);
            }
        };

        BotState {
            exchange_storage: Arc::new(ExchangeRepository::new(pool)),
        }
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::exchange()],
            on_error: |error| {
                Box::pin(async move {
                    handle_error(error).await;
                })
            },
            ..Default::default()
        })
        .token(app_config.discord_bot_token)
        .intents(serenity::GatewayIntents::GUILD_MESSAGES)
        .setup(move |ctx, _ready, framework| {
            Box::pin(
                async move {
                    let commands = &framework.options().commands;

                    if let Some(true) = app_config.register_commands_globally {
                        info!("Registering commands globally");
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                    }

                    if let Some(guilds) = app_config.register_commands_in_guilds {
                        for guild in guilds.iter().map(|g| GuildId(*g)) {
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

                    AssignmentService::create_and_start(app_state.exchange_storage.clone());

                    Ok(app_state)
                }
                .instrument(info_span!("bot_setup")),
            )
        });

    framework.run().await.unwrap();
}

#[tracing::instrument(skip(url))]
async fn setup_database(url: &str) -> anyhow::Result<SqlitePool> {
    info!("Connecting to SQLite database at {url}");
    let pool = SqlitePoolOptions::new().connect(url).await?;
    info!("Running migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Done!");
    Ok(pool)
}
