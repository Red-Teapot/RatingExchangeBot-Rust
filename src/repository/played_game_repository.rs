use sqlx::{Pool, Sqlite};

pub struct PlayedGameRepository {
    pool: Pool<Sqlite>,
}

impl PlayedGameRepository {
    pub fn new(pool: Pool<Sqlite>) -> PlayedGameRepository {
        PlayedGameRepository { pool }
    }
}
