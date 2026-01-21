//! 用户会话数据模型

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// 用户会话
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserSession {
    /// 根据 ID 获取会话
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM user_sessions WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 根据 token 哈希获取会话
    pub async fn get_by_token_hash(
        pool: &Pool<Sqlite>,
        token_hash: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM user_sessions WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(pool)
            .await
    }

    /// 获取有效会话（未过期）
    pub async fn get_valid_by_token_hash(
        pool: &Pool<Sqlite>,
        token_hash: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let now = Utc::now();
        sqlx::query_as::<_, Self>(
            "SELECT * FROM user_sessions WHERE token_hash = ? AND expires_at > ?",
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(pool)
        .await
    }

    /// 获取用户的所有会话
    pub async fn list_by_user(pool: &Pool<Sqlite>, user_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM user_sessions WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// 创建会话
    pub async fn create(
        pool: &Pool<Sqlite>,
        id: &str,
        user_id: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, expires_at, created_at, ip_address, user_agent)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(now)
        .bind(ip_address)
        .bind(user_agent)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 延长会话有效期
    pub async fn extend(
        pool: &Pool<Sqlite>,
        id: &str,
        new_expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE user_sessions SET expires_at = ? WHERE id = ?")
            .bind(new_expires_at)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 删除会话（登出）
    pub async fn delete(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_sessions WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 根据 token 哈希删除会话
    pub async fn delete_by_token_hash(
        pool: &Pool<Sqlite>,
        token_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_sessions WHERE token_hash = ?")
            .bind(token_hash)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除用户的所有会话
    pub async fn delete_all_by_user(pool: &Pool<Sqlite>, user_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 清理过期会话
    pub async fn cleanup_expired(pool: &Pool<Sqlite>) -> Result<u64, sqlx::Error> {
        let now = Utc::now();
        let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at <= ?")
            .bind(now)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 限制用户会话数量（删除最旧的会话）
    pub async fn limit_user_sessions(
        pool: &Pool<Sqlite>,
        user_id: &str,
        max_sessions: u32,
    ) -> Result<u64, sqlx::Error> {
        // 删除超出限制的最旧会话
        let result = sqlx::query(
            r#"
            DELETE FROM user_sessions
            WHERE id IN (
                SELECT id FROM user_sessions
                WHERE user_id = ?
                ORDER BY created_at DESC
                LIMIT -1 OFFSET ?
            )
            "#,
        )
        .bind(user_id)
        .bind(max_sessions)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// 统计用户会话数量
    pub async fn count_by_user(pool: &Pool<Sqlite>, user_id: &str) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }

    /// 检查会话是否过期
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    /// 检查会话是否需要刷新（剩余时间少于指定秒数）
    pub fn needs_refresh(&self, threshold_secs: i64) -> bool {
        let threshold = Utc::now() + Duration::seconds(threshold_secs);
        self.expires_at <= threshold
    }
}
