use serenity::all::UserId;
use sqlx::{query, query_as, Pool, Sqlite};

use crate::{
    models::{ExchangeId, Submission, SubmissionId},
    repository::submission_repository::SqlSubmission,
};

use super::conversion::DBConvertible;

#[derive(Debug)]
pub struct AssignmentRepository {
    pool: Pool<Sqlite>,
}

impl AssignmentRepository {
    pub fn new(pool: Pool<Sqlite>) -> AssignmentRepository {
        AssignmentRepository { pool }
    }

    pub async fn add_assignment(
        &self,
        exchange_id: ExchangeId,
        submission_id: SubmissionId,
        submitter: UserId,
    ) -> Result<(), anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        {
            let exchange_id = exchange_id.to_db()?;
            let submission_id = submission_id.to_db()?;
            let submitter = submitter.to_db()?;

            query!(
                r#"
                    INSERT INTO assignments (exchange_id, submission_id, submitter)
                    VALUES ($1, $2, $3)
                "#,
                exchange_id,
                submission_id,
                submitter,
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_assignments(
        &self,
        exchange_id: ExchangeId,
        user_id: UserId,
    ) -> Result<Vec<Submission>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let assignments = {
            let exchange_id = exchange_id.to_db()?;
            let user_id = user_id.to_db()?;

            query_as!(
                SqlSubmission,
                r#"
                SELECT submissions.* FROM submissions
                INNER JOIN assignments ON submissions.id = assignments.submission_id
                WHERE assignments.exchange_id = $1 AND assignments.submitter = $2
                "#,
                exchange_id,
                user_id,
            )
            .fetch_all(&mut *transaction)
            .await?
            .iter()
            .map(Submission::from_db)
            .collect::<Result<Vec<Submission>, _>>()?
        };

        transaction.commit().await?;

        Ok(assignments)
    }
}
