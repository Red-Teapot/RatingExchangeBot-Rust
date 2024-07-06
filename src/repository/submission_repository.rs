use poise::serenity_prelude::UserId;
use sqlx::{query_as, Pool, Sqlite};
use tracing::debug;

use crate::{
    models::{types::UtcDateTime, ExchangeId, NewSubmission, Submission, SubmissionId},
    repository::conversion::DBConvertible,
};

use super::conversion::{DBFromConversionError, DBToConversionError};

pub struct SubmissionRepository {
    pool: Pool<Sqlite>,
}

impl SubmissionRepository {
    pub fn new(pool: Pool<Sqlite>) -> SubmissionRepository {
        SubmissionRepository { pool }
    }

    pub async fn get_conflicting_submission(
        &self,
        exchange_id: ExchangeId,
        submitter: UserId,
        link: &str,
    ) -> Result<Option<Submission>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let conflict = {
            let exchange_id = exchange_id.to_db()?;
            let submitter = submitter.to_db()?;

            query_as!(
                SqlSubmission,
                r#"
                    SELECT * FROM submissions
                    WHERE exchange_id = $1 AND (submitter = $2 OR link = $3)
                "#,
                exchange_id,
                submitter,
                link,
            )
            .fetch_optional(&mut transaction)
            .await?
        };

        transaction.commit().await?;

        debug!("Conflict for exchange {exchange_id:?}, submitter {submitter}, link {link}: {conflict:?}");

        match conflict {
            Some(conflict) => Ok(Some(Submission::from_db(&conflict)?)),
            None => Ok(None),
        }
    }

    pub async fn add_or_update_submission(
        &self,
        submission: &NewSubmission,
    ) -> Result<Submission, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let added_submission = {
            let exchange_id = submission.exchange_id.to_db()?;
            let link = &submission.link;
            let submitter = submission.submitter.to_db()?;
            let submitted_at = submission.submitted_at.to_db()?;

            query_as!(
                SqlSubmission,
                r#"
                    INSERT INTO submissions (exchange_id, link, submitter, submitted_at)
                    VALUES ($1, $2, $3, $4)
                    RETURNING 
                        id AS "id!", 
                        exchange_id AS "exchange_id!",
                        link AS "link!",
                        submitter AS "submitter!",
                        submitted_at AS "submitted_at!"
                "#,
                exchange_id,
                link,
                submitter,
                submitted_at,
            )
            .fetch_one(&mut transaction)
            .await?
        };

        transaction.commit().await?;

        debug!("New submission: {added_submission:?}");

        Ok(Submission::from_db(&added_submission)?)
    }
}

#[derive(Debug)]
pub struct SqlSubmission {
    id: i64,
    exchange_id: i64,
    link: String,
    submitter: i64,
    submitted_at: String,
}

impl DBConvertible for Submission {
    type DBType = SqlSubmission;

    fn to_db(&self) -> Result<Self::DBType, DBToConversionError> {
        Ok(SqlSubmission {
            id: self.id.to_db()?,
            exchange_id: self.exchange_id.to_db()?,
            link: self.link.clone(),
            submitter: self.submitter.to_db()?,
            submitted_at: self.submitted_at.to_db()?,
        })
    }

    fn from_db(value: &Self::DBType) -> Result<Self, DBFromConversionError> {
        Ok(Submission {
            id: SubmissionId::from_db(&value.id)?,
            exchange_id: ExchangeId::from_db(&value.exchange_id)?,
            link: value.link.clone(),
            submitter: UserId::from_db(&value.submitter)?,
            submitted_at: UtcDateTime::from_db(&value.submitted_at)?,
        })
    }
}

#[cfg(test)]
mod test {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::SubmissionRepository;

    async fn create_inmem_repository() -> Result<SubmissionRepository, anyhow::Error> {
        let pool = SqlitePoolOptions::new().connect("file::memory:").await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        let repository = SubmissionRepository { pool };

        Ok(repository)
    }

    #[tokio::test]
    async fn nothing() {
        // let repository = create_inmem_repository().await.unwrap();
    }
}
