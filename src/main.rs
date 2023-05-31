#![forbid(unsafe_code)]
#![allow(dead_code)] // TODO: Remove this before the first release.

mod actors;
mod assignment_service;
mod commands;
mod data;
mod env_vars;
mod jam_types;
mod solver;
mod storage;
mod utils;

use std::sync::Arc;

use assignment_service::AssignmentService;
use log::*;

use poise::serenity_prelude::{self as serenity, GuildId};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use storage::ExchangeStorage;

pub struct BotState {
    pub exchange_storage: Arc<ExchangeStorage>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(err) = envmnt::load_file(".env") {
        warn!("Could not load .env file: {err}");
    }

    if !env_vars::check() {
        std::process::exit(255);
    }

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
            ..Default::default()
        })
        .token(env_vars::DISCORD_BOT_TOKEN.required())
        .intents(serenity::GatewayIntents::GUILD_MESSAGES)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let commands = &framework.options().commands;

                if env_vars::REGISTER_COMMANDS_GLOBALLY.get_bool(false) {
                    info!("Registering commands globally");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
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
            })
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
