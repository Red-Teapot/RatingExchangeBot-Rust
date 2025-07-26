mod create;
mod delete;
mod list;

use tracing::error;

use super::{CommandError, CommandResult, Context, user_err};

#[poise::command(
    slash_command,
    guild_only,
    subcommands("create::create", "list::list", "delete::delete"),
    check = check_manager_role,
)]
pub async fn exchange(_ctx: Context<'_>) -> CommandResult {
    Err(user_err("The `/exchange` command is not supported yet"))
}

async fn check_manager_role(ctx: Context<'_>) -> Result<bool, CommandError> {
    let result = check_manager_role_inner(ctx).await;

    if let Err(err) = &result {
        error!("Could not check manager role: {err:?}");
    }

    result.map_err(|err| CommandError::Internal {
        message: err.to_string(),
    })
}

async fn check_manager_role_inner(ctx: Context<'_>) -> anyhow::Result<bool> {
    let guild_id = match ctx.guild_id() {
        Some(guild_id) => guild_id,
        None => return Ok(false),
    };
    let user = ctx.author();

    let member = guild_id.member(&ctx, user.id).await?;

    ctx.data()
        .permission_service
        .check_permissions(guild_id, &member)
        .await
}
