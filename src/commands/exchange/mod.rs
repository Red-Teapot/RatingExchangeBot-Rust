mod create;
mod delete;
mod list;

use serenity::all::Permissions;
use tracing::{debug, error, warn};

use crate::repository::ManagerRole;

use super::{user_err, CommandError, CommandResult, Context};

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

    result
}

async fn check_manager_role_inner(ctx: Context<'_>) -> Result<bool, CommandError> {
    let guild_id = match ctx.guild_id() {
        Some(guild_id) => guild_id,
        None => return Ok(false),
    };
    let user = ctx.author();

    debug!("Checking user: {user:?}");

    let member = guild_id.member(&ctx, user.id).await?;

    debug!("Member info: {member:?}");

    let guild_owner = ctx.partial_guild().await.map(|g| g.owner_id);

    if let Some(guild_owner) = guild_owner {
        if guild_owner == user.id {
            debug!("Manager role check passed - user is the guild owner");
            return Ok(true);
        }
    }

    for role_id in member.roles.iter() {
        match guild_id.role(&ctx, *role_id).await {
            Ok(role) => {
                if role.has_permission(Permissions::ADMINISTRATOR) {
                    debug!("Manager role check passed - user is an admin");
                    return Ok(true);
                }
            }
            Err(err) => {
                warn!("Could not get role {role_id}: {err}");
            }
        }
    }

    let setting_repository = ctx.framework().user_data.setting_repository.clone();
    let manager_role = match setting_repository.get::<ManagerRole>(guild_id).await {
        Ok(Some(manager_role)) => manager_role.role,
        _ => {
            debug!("Manager role check failed - manager role is not configured");
            return Ok(false);
        }
    };

    if user.has_role(ctx, guild_id, manager_role).await? {
        debug!("Manager role check passed - user is a manager");
        Ok(true)
    } else {
        debug!("Manager role check failed - user is not a manager");
        Ok(false)
    }
}
