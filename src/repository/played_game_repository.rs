use serenity::all::UserId;
use sqlx::{query, query_as, Pool, Sqlite};

use crate::models::{ExchangeId, PlayedGame, PlayedGameId};

use super::conversion::DBConvertible;

pub struct PlayedGameRepository {
    pool: Pool<Sqlite>,
}

impl PlayedGameRepository {
    pub fn new(pool: Pool<Sqlite>) -> PlayedGameRepository {
        PlayedGameRepository { pool }
    }

    pub async fn submit(&self, user: UserId, link: &str) -> Result<(), anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let user = user.to_db()?;
        query!(
            r#"
                INSERT INTO played_games (member, link, is_manual)
                VALUES ($1, $2, TRUE)
                ON CONFLICT (member, link) DO NOTHING
            "#,
            user,
            link,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get_played_games_for_exchange(
        &self,
        exchange_id: ExchangeId,
    ) -> Result<Vec<PlayedGame>, anyhow::Error> {
        let mut transaction = self.pool.begin().await?;

        let played_games = {
            let exchange_id = exchange_id.to_db()?;

            query_as!(
                SqlPlayedGame,
                r#"
                    SELECT played_games.* FROM played_games
                    INNER JOIN submissions ON submissions.submitter = played_games.member
                    WHERE submissions.exchange_id = $1
                "#,
                exchange_id,
            )
            .fetch_all(&mut *transaction)
            .await?
            .iter()
            .map(|s| PlayedGame::from_db(s))
            .collect::<Result<Vec<PlayedGame>, _>>()?
        };

        transaction.commit().await?;

        Ok(played_games)
    }
}

#[derive(Debug)]
pub struct SqlPlayedGame {
    pub id: i64,
    pub link: String,
    pub member: i64,
    pub is_manual: i64,
}

impl DBConvertible for PlayedGame {
    type DBType = SqlPlayedGame;

    fn to_db(&self) -> Result<Self::DBType, super::conversion::DBToConversionError> {
        Ok(SqlPlayedGame {
            id: self.id.to_db()?,
            link: self.link.clone(),
            member: self.member.to_db()?,
            is_manual: if self.is_manual { 1 } else { 0 },
        })
    }

    fn from_db(value: &Self::DBType) -> Result<Self, super::conversion::DBFromConversionError> {
        Ok(PlayedGame {
            id: PlayedGameId::from_db(&value.id)?,
            link: value.link.clone(),
            member: UserId::from_db(&value.member)?,
            is_manual: value.is_manual > 0,
        })
    }
}

#[cfg(test)]
mod test {
    use serenity::all::UserId;
    use sqlx::{query, sqlite::SqlitePoolOptions, SqlitePool};

    use crate::{
        models::{ExchangeId, PlayedGame, PlayedGameId},
        repository::PlayedGameRepository,
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
    async fn no_played_games() {
        let pool = setup_database().await;
        let repository = PlayedGameRepository::new(pool.clone());

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
                    
                    INSERT INTO played_games (id, member, link, is_manual)
                    VALUES (1, 7, 'https://itch.io/jam/example-jam-2/rate/000002', FALSE),
                           (2, 8, 'https://itch.io/jam/example-jam-2/rate/000003', FALSE),
                           (3, 9, 'https://itch.io/jam/example-jam-2/rate/000004', FALSE);
                "#
            ).execute(&mut *transaction).await.unwrap();

            transaction.commit().await.unwrap();
        };

        let played_games = repository
            .get_played_games_for_exchange(ExchangeId(1))
            .await
            .unwrap();

        assert!(played_games.is_empty());
    }

    #[tokio::test]
    async fn one_played_game() {
        let pool = setup_database().await;
        let repository = PlayedGameRepository::new(pool.clone());

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
                    
                    INSERT INTO played_games (id, member, link, is_manual)
                    VALUES (1, 7, 'https://itch.io/jam/example-jam-2/rate/000002', FALSE),
                           (2, 8, 'https://itch.io/jam/example-jam-2/rate/000003', TRUE),
                           (3, 9, 'https://itch.io/jam/example-jam-2/rate/000004', FALSE);
                "#
            ).execute(&mut *transaction).await.unwrap();

            transaction.commit().await.unwrap();
        };

        let played_games = repository
            .get_played_games_for_exchange(ExchangeId(1))
            .await
            .unwrap();

        assert_eq!(
            played_games,
            vec![PlayedGame {
                id: PlayedGameId(3),
                link: "https://itch.io/jam/example-jam-2/rate/000004".to_string(),
                member: UserId::new(9),
                is_manual: false,
            }]
        );
    }

    #[tokio::test]
    async fn multiple_played_games() {
        let pool = setup_database().await;
        let repository = PlayedGameRepository::new(pool.clone());

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
                    
                    INSERT INTO played_games (id, member, link, is_manual)
                    VALUES (1, 7, 'https://itch.io/jam/example-jam-2/rate/000002', FALSE),
                           (2, 8, 'https://itch.io/jam/example-jam-2/rate/000003', TRUE),
                           (3, 9, 'https://itch.io/jam/example-jam-2/rate/000004', FALSE);
                "#
            ).execute(&mut *transaction).await.unwrap();

            transaction.commit().await.unwrap();
        };

        let played_games = repository
            .get_played_games_for_exchange(ExchangeId(4))
            .await
            .unwrap();

        assert_eq!(
            played_games,
            vec![
                PlayedGame {
                    id: PlayedGameId(1),
                    link: "https://itch.io/jam/example-jam-2/rate/000002".to_string(),
                    member: UserId::new(7),
                    is_manual: false,
                },
                PlayedGame {
                    id: PlayedGameId(2),
                    link: "https://itch.io/jam/example-jam-2/rate/000003".to_string(),
                    member: UserId::new(8),
                    is_manual: true,
                },
            ]
        );
    }
}
