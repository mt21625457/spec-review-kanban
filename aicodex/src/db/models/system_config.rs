use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// 配置值类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigValueType {
    String,
    Integer,
    Boolean,
    Json,
}

impl Default for ConfigValueType {
    fn default() -> Self {
        Self::String
    }
}

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub id: String,
    pub config_key: String,
    pub config_value: String,
    pub is_encrypted: bool,
    pub value_type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 配置键常量
pub mod keys {
    // Gitea 配置
    pub const GITEA_API_URL: &str = "gitea.api_url";
    pub const GITEA_TOKEN: &str = "gitea.token";
    pub const GITEA_WEBHOOK_SECRET: &str = "gitea.webhook_secret";

    // 审核配置
    pub const REVIEW_DEFAULT_AGENT: &str = "review.default_agent";
    pub const REVIEW_AUTO_START: &str = "review.auto_start";
    pub const REVIEW_TIMEOUT_SECONDS: &str = "review.timeout_seconds";

    // 队列配置
    pub const QUEUE_MAX_CONCURRENT: &str = "queue.max_concurrent";
    pub const QUEUE_RETRY_COUNT: &str = "queue.retry_count";
    pub const QUEUE_RETRY_DELAY_SECONDS: &str = "queue.retry_delay_seconds";
}

impl SystemConfig {
    /// 获取配置值
    pub async fn get(pool: &Pool<Sqlite>, key: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM system_configs WHERE config_key = ?"
        )
        .bind(key)
        .fetch_optional(pool)
        .await
    }

    /// 设置配置值
    pub async fn set(
        pool: &Pool<Sqlite>,
        key: &str,
        value: &str,
        is_encrypted: bool,
        value_type: ConfigValueType,
        description: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let value_type_str = match value_type {
            ConfigValueType::String => "string",
            ConfigValueType::Integer => "integer",
            ConfigValueType::Boolean => "boolean",
            ConfigValueType::Json => "json",
        };

        sqlx::query(
            r#"
            INSERT INTO system_configs (id, config_key, config_value, is_encrypted, value_type, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(config_key) DO UPDATE SET
                config_value = excluded.config_value,
                is_encrypted = excluded.is_encrypted,
                value_type = excluded.value_type,
                description = COALESCE(excluded.description, system_configs.description),
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&id)
        .bind(key)
        .bind(value)
        .bind(is_encrypted)
        .bind(value_type_str)
        .bind(description)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get(pool, key).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 删除配置
    pub async fn delete(pool: &Pool<Sqlite>, key: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM system_configs WHERE config_key = ?")
            .bind(key)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 列出所有配置
    pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM system_configs ORDER BY config_key")
            .fetch_all(pool)
            .await
    }

    /// 按前缀列出配置
    pub async fn list_by_prefix(pool: &Pool<Sqlite>, prefix: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM system_configs WHERE config_key LIKE ? ORDER BY config_key"
        )
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await
    }
}
