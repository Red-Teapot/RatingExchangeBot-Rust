mod create;
mod delete;
mod list;

use super::{user_err, CommandResult, Context};

#[poise::command(
    slash_command,
    guild_only,
    subcommands("create::create", "list::list", "delete::delete"),
    required_permissions = "ADMINISTRATOR",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn exchange(_ctx: Context<'_>) -> CommandResult {
    Err(user_err("The `/exchange` command is not supported yet"))
}
