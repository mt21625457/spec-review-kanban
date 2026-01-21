pub mod models;

use sqlx::{Pool, Sqlite};

/// 数据库封装
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}
