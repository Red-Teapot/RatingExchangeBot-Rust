#![forbid(unsafe_code)]
#![allow(dead_code)] // TODO: Remove this before the first release.

use log::*;

use poise::serenity_prelude::{self as serenity, GuildId};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tiny_tokio_actor::*;

mod commands;
mod data;
mod env_vars;
mod jam_types;
mod solver;
mod storage;

pub struct BotState {}

#[derive(Clone)]
struct RebotSystemEvent {}
impl SystemEvent for RebotSystemEvent {}

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(err) = envmnt::load_file(".env") {
        warn!("Could not load .env file: {err}");
    }

    if !env_vars::check() {
        std::process::exit(255);
    }

    let _pool = match setup_database().await {
        Ok(pool) => pool,
        Err(err) => {
            error!("Could not setup database: {err}");
            std::process::exit(255);
        }
    };

    let bus = EventBus::<RebotSystemEvent>::new(512);
    let _system = ActorSystem::new("rebot", bus);

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

                Ok(BotState {})
            })
        });

    framework.run().await.unwrap();
}

async fn setup_database() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .connect(&env_vars::DATABASE_URL.required())
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
