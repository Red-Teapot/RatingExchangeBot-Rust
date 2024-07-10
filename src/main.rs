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

use poise::{serenity_prelude::*, Framework};
use poise_error_handler::handle_error;
use repository::{ExchangeRepository, PlayedGameRepository, SubmissionRepository};
use serde::Deserialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tokio::{select, signal, sync::Notify};
use tracing::{error, info, info_span, warn, Instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct AppConfig {
    discord_bot_token: String,
    database_url: String,
    register_commands_globally: Option<bool>,
    register_commands_in_guilds: Option<Vec<u64>>,
}

pub struct BotState {
    pub exchange_repository: Arc<ExchangeRepository>,
    pub submission_repository: Arc<SubmissionRepository>,
    pub played_game_repository: Arc<PlayedGameRepository>,
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

    let db_pool = match setup_database(&app_config.database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            error!("Could not setup database: {err}");
            std::process::exit(255);
        }
    };

    let shutdown_notify = Arc::new(Notify::new());
    let assignment_service_shutdown = shutdown_notify.clone();

    let app_state = BotState {
        exchange_repository: Arc::new(ExchangeRepository::new(db_pool.clone())),
        submission_repository: Arc::new(SubmissionRepository::new(db_pool.clone())),
        played_game_repository: Arc::new(PlayedGameRepository::new(db_pool.clone())),
    };

    let framework = Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::exchange(), commands::submit()],
            on_error: |error| Box::pin(handle_error(error)),
            ..Default::default()
        })
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
                        for guild in guilds.iter().map(|g| GuildId::new(*g)) {
                            let guild_name = ctx
                                .http()
                                .get_guild(guild)
                                .await
                                .map(|g| g.name)
                                .unwrap_or("???".to_string());

                            info!("Registering commands in guild {guild} ({guild_name})");

                            poise::builtins::register_in_guild(ctx, commands, guild).await?;
                        }
                    }

                    AssignmentService::create_and_start(
                        assignment_service_shutdown,
                        ctx.http.clone(),
                        app_state.exchange_repository.clone(),
                        app_state.submission_repository.clone(),
                        app_state.played_game_repository.clone(),
                    );

                    Ok(app_state)
                }
                .instrument(info_span!("bot_setup")),
            )
        })
        .build();

    let mut client = match ClientBuilder::new(app_config.discord_bot_token, GatewayIntents::empty())
        .framework(framework)
        .await
    {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to create the client: {err}");
            exit(255);
        }
    };

    select! {
        _ = signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down");
            shutdown_notify.notify_waiters();
            client.shard_manager.shutdown_all().await;
            db_pool.close().await;
        },

        result = client.start() => {
            if let Err(err) = result {
                error!("Failed to start the client: {err}");
            }
        },
    };
}

async fn run() -> Result<(), anyhow::Error> {
    Ok(())
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
