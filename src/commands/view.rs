use indoc::formatdoc;
use poise::CreateReply;
use tracing::warn;

use crate::commands::{internal_err, user_err, ApplicationContext};

use super::CommandResult;

#[poise::command(slash_command, rename = "view")]
pub async fn view(ctx: ApplicationContext<'_>, exchange_slug: String) -> CommandResult {
    let exchange = match ctx
        .data
        .exchange_repository
        .get_exchange_by_slug(&exchange_slug)
        .await
    {
        Ok(exchange) => exchange,
        Err(err) => {
            warn!("Could not find the exchange for slug {exchange_slug}: {err}");
            return Err(user_err("Could not find the exchange"));
        }
    };

    let user_id = ctx.author().id;

    let assignments = match ctx
        .data
        .assignment_repository
        .get_assignments(exchange.id, user_id)
        .await
    {
        Ok(assignments) => assignments,
        Err(err) => {
            warn!(
                "Could not get assignments for exchange {exchange:?} and user {user_id:?}: {err}"
            );
            return Err(internal_err("Could not get assignments"));
        }
    };

    let assignments_str = &assignments
        .iter()
        .map(|assignment| format!(" - {}", assignment.link))
        .collect::<Vec<String>>()
        .join("\n");

    let message = if assignments.is_empty() {
        formatdoc! {
            r#"
                # No assignments found for {exchange_name}

                Looks like there are no assignments for you in that exchange
            "#,
            exchange_name = exchange.display_name,
        }
    } else {
        formatdoc! {
            r#"
                # Here are your assignments for {exchange_name}

                {assignments_str}
            "#,
            exchange_name = exchange.display_name,
            assignments_str = assignments_str,
        }
    };

    ctx.send(CreateReply::default().ephemeral(true).content(message))
        .await?;

    Ok(())
}
