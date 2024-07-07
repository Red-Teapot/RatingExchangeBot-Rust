use poise::CreateReply;

use crate::commands::{internal_err, user_err, ApplicationContext, CommandResult};

#[poise::command(slash_command, rename = "delete")]
pub async fn delete(
    ctx: ApplicationContext<'_>,
    #[description = "Exchange slug"] slug: String,
) -> CommandResult {
    let deletion_result = ctx
        .data
        .exchange_repository
        .delete_exchange(
            ctx.guild_id().ok_or(internal_err(
                "This command should be executed only in a guild",
            ))?,
            &slug,
        )
        .await;

    match deletion_result {
        Ok(true) => {
            ctx.send(CreateReply::default().content(format!("# Exchange `{slug}` deleted")))
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
