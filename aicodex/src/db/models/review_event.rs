use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// 审核事件
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewEvent {
    pub id: String,
    pub review_run_id: String,
    pub event_type: String,
    pub event_data: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ReviewEvent {
    /// 创建事件
    pub async fn create(
        pool: &Pool<Sqlite>,
        review_run_id: &str,
        event_type: &str,
        event_data: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO review_events (id, review_run_id, event_type, event_data, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(review_run_id)
        .bind(event_type)
        .bind(event_data)
        .bind(now)
        .execute(pool)
        .await?;

        sqlx::query_as::<_, Self>("SELECT * FROM review_events WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await
    }

    /// 列出审核运行的所有事件
    pub async fn list_for_review_run(pool: &Pool<Sqlite>, review_run_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM review_events WHERE review_run_id = ? ORDER BY created_at ASC"
        )
        .bind(review_run_id)
        .fetch_all(pool)
        .await
    }

    /// 列出指定时间之后的事件
    pub async fn list_since(
        pool: &Pool<Sqlite>,
        review_run_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM review_events
            WHERE review_run_id = ? AND created_at > ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(review_run_id)
        .bind(since)
        .fetch_all(pool)
        .await
    }
}
