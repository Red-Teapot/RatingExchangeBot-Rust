use indoc::formatdoc;
use poise::CreateReply;
use strum::IntoEnumIterator;

use crate::{
    commands::{user_err, ApplicationContext, CommandResult},
    jam_types::JamType,
};

use super::internal_err;

#[poise::command(slash_command, rename = "played")]
pub async fn played(
    ctx: ApplicationContext<'_>,
    #[description = "Submission link"] link: String,
) -> CommandResult {
    let user = ctx.author().id;

    if JamType::iter().all(|jam_type| !jam_type.validate_entry_link(&link)) {
        return Err(user_err(
            "Invalid entry link, does not match any of known jams",
        ));
    }

    match ctx
        .data
        .played_game_repository
        .submit(user, &link, true)
        .await
    {
        Ok(_) => {
            let message = formatdoc! {
                r#"
                    # Registered this submission as played!

                    You won't be assigned this submission in future exchanges.
                "#,
            };
            ctx.send(CreateReply::default().ephemeral(true).content(message))
                .await?;
            Ok(())
        }
        Err(err) => Err(internal_err(format!(
            "Could not register the game as played: {err}"
        ))),
    }
}
