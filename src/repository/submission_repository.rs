use poise::serenity_prelude::UserId;
use sqlx::{query, query_as, Pool, Sqlite};

use crate::{
    models::{
        types::UtcDateTime, ExchangeId, ExchangeState, NewSubmission, Submission, SubmissionId,
    },
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
        new_submission: &NewSubmission,
    ) -> Result<Option<Submission>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let conflict = {
            let exchange_id = new_submission.exchange_id.to_db()?;
            let submitter = new_submission.submitter.to_db()?;

            query_as!(
                SqlSubmission,
                r#"
                    SELECT * FROM submissions
                    WHERE exchange_id = $1 AND (submitter = $2 OR link = $3)
                    LIMIT 1
                "#,
                exchange_id,
                submitter,
                new_submission.link,
            )
            .fetch_optional(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

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
                    ON CONFLICT (exchange_id, submitter) DO UPDATE SET link = $2
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
            .fetch_one(&mut *transaction)
            .await?
        };

        transaction.commit().await?;

        Ok(Submission::from_db(&added_submission)?)
    }

    pub async fn revoke(
        &self,
        exchange_id: ExchangeId,
        submitter: UserId,
    ) -> Result<bool, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let exchange_id = exchange_id.to_db()?;
        let submitter = submitter.to_db()?;
        let accepting_submissions = ExchangeState::AcceptingSubmissions.to_db()?;
        let result = query!(
            r#"
                DELETE FROM submissions
                WHERE exchange_id = $1 AND submitter = $2
                    AND EXISTS(SELECT 1 FROM exchanges 
                        WHERE submissions.exchange_id = exchanges.id AND exchanges.state = $3)
            "#,
            exchange_id,
            submitter,
            accepting_submissions,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_submissions_for_exchange(
        &self,
        exchange_id: ExchangeId,
    ) -> Result<Vec<Submission>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let submissions = {
            let exchange_id = exchange_id.to_db()?;

            query_as!(
                SqlSubmission,
                r#"
                    SELECT * FROM submissions WHERE exchange_id = $1
                "#,
                exchange_id,
            )
            .fetch_all(&mut *transaction)
            .await?
            .iter()
            .map(|s| Submission::from_db(s))
            .collect::<Result<Vec<Submission>, _>>()?
        };

        transaction.commit().await?;

        Ok(submissions)
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
    use serenity::all::UserId;
    use sqlx::{query, sqlite::SqlitePoolOptions, SqlitePool};
    use time::macros::datetime;

    use crate::{
        models::{types::UtcDateTime, ExchangeId, Submission, SubmissionId},
        repository::SubmissionRepository,
    };

    async fn setup_database() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn no_submissions() {
        let pool = setup_database().await;
        let repository = SubmissionRepository::new(pool.clone());

        {
            let mut transaction = pool.begin().await.unwrap();

            query!(
                r#"
                    INSERT INTO exchanges (id, guild, channel, jam_type, jam_link, slug, display_name, state, submissions_start, submissions_end, games_per_member) 
                    VALUES (1, 2, 3, 'Itch', 'https://itch.io/jam/example-jam', 'Test', 'Test', 'AcceptingSubmissions', '2024-01-01T00:00:00.000000000Z', '2024-01-02T00:00:00.000000000Z', 5),
                           (4, 5, 6, 'Itch', 'https://itch.io/jam/example-jam-2', 'Test2', 'Test 2', 'AcceptingSubmissions', '2024-01-01T00:00:00.000000000Z', '2024-01-02T00:00:00.000000000Z', 5);

                    INSERT INTO submissions (id, exchange_id, link, submitter, submitted_at)
                    VALUES (1, 4, 'https://itch.io/jam/example-jam-2/rate/000004', 7, '2024-01-01T00:01:00.000000000Z'),
                           (2, 4, 'https://itch.io/jam/example-jam-2/rate/000005', 8, '2024-01-01T00:01:00.000000000Z');
                "#
            ).execute(&mut *transaction).await.unwrap();

            transaction.commit().await.unwrap();
        };

        let submissions = repository
            .get_submissions_for_exchange(ExchangeId(1))
            .await
            .unwrap();

        assert!(submissions.is_empty());
    }

    #[tokio::test]
    async fn one_submission() {
        let pool = setup_database().await;
        let repository = SubmissionRepository::new(pool.clone());

        {
            let mut transaction = pool.begin().await.unwrap();

            query!(
                r#"
                    INSERT INTO exchanges (id, guild, channel, jam_type, jam_link, slug, display_name, state, submissions_start, submissions_end, games_per_member) 
                    VALUES (1, 2, 3, 'Itch', 'https://itch.io/jam/example-jam', 'Test', 'Test', 'AcceptingSubmissions', '2024-01-01T00:00:00.000000000Z', '2024-01-02T00:00:00.000000000Z', 5),
                           (4, 5, 6, 'Itch', 'https://itch.io/jam/example-jam-2', 'Test2', 'Test 2', 'AcceptingSubmissions', '2024-01-01T00:00:00.000000000Z', '2024-01-02T00:00:00.000000000Z', 5);

                    INSERT INTO submissions (id, exchange_id, link, submitter, submitted_at)
                    VALUES (1, 4, 'https://itch.io/jam/example-jam-2/rate/000004', 7, '2024-01-01T00:01:00.000000000Z'),
                           (2, 4, 'https://itch.io/jam/example-jam-2/rate/000005', 8, '2024-01-01T00:01:00.000000000Z'),
                           (3, 1, 'https://itch.io/jam/example-jam/rate/000003', 9, '2024-01-01T00:01:00.000000000Z');
                "#
            ).execute(&mut *transaction).await.unwrap();

            transaction.commit().await.unwrap();
        };

        let submissions = repository
            .get_submissions_for_exchange(ExchangeId(1))
            .await
            .unwrap();

        assert_eq!(
            submissions,
            vec![Submission {
                id: SubmissionId(3),
                exchange_id: ExchangeId(1),
                link: "https://itch.io/jam/example-jam/rate/000003".to_string(),
                submitter: UserId::new(9),
                submitted_at: UtcDateTime::assume_utc(datetime!(2024-01-01 00:01:00.000000000)),
            }]
        );
    }

    #[tokio::test]
    async fn multiple_submissions() {
        let pool = setup_database().await;
        let repository = SubmissionRepository::new(pool.clone());

        {
            let mut transaction = pool.begin().await.unwrap();

            query!(
                r#"
                    INSERT INTO exchanges (id, guild, channel, jam_type, jam_link, slug, display_name, state, submissions_start, submissions_end, games_per_member) 
                    VALUES (1, 2, 3, 'Itch', 'https://itch.io/jam/example-jam', 'Test', 'Test', 'AcceptingSubmissions', '2024-01-01T00:00:00.000000000Z', '2024-01-02T00:00:00.000000000Z', 5),
                           (4, 5, 6, 'Itch', 'https://itch.io/jam/example-jam-2', 'Test2', 'Test 2', 'AcceptingSubmissions', '2024-01-01T00:00:00.000000000Z', '2024-01-02T00:00:00.000000000Z', 5);

                    INSERT INTO submissions (id, exchange_id, link, submitter, submitted_at)
                    VALUES (1, 4, 'https://itch.io/jam/example-jam-2/rate/000004', 7, '2024-01-01T00:01:00.000000000Z'),
                           (2, 4, 'https://itch.io/jam/example-jam-2/rate/000005', 8, '2024-01-01T00:01:00.000000000Z'),
                           (3, 1, 'https://itch.io/jam/example-jam/rate/000003', 9, '2024-01-01T00:01:00.000000000Z');
                "#
            ).execute(&mut *transaction).await.unwrap();

            transaction.commit().await.unwrap();
        };

        let submissions = repository
            .get_submissions_for_exchange(ExchangeId(4))
            .await
            .unwrap();

        assert_eq!(
            submissions,
            vec![
                Submission {
                    id: SubmissionId(1),
                    exchange_id: ExchangeId(4),
                    link: "https://itch.io/jam/example-jam-2/rate/000004".to_string(),
                    submitter: UserId::new(7),
                    submitted_at: UtcDateTime::assume_utc(datetime!(2024-01-01 00:01:00.000000000)),
                },
                Submission {
                    id: SubmissionId(2),
                    exchange_id: ExchangeId(4),
                    link: "https://itch.io/jam/example-jam-2/rate/000005".to_string(),
                    submitter: UserId::new(8),
                    submitted_at: UtcDateTime::assume_utc(datetime!(2024-01-01 00:01:00.000000000)),
                }
            ]
        );
    }
}
