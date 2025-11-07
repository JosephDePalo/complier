pub mod models;

use sqlx::PgPool;

pub struct Db {
    pub pool: PgPool,
}

impl Db {}
