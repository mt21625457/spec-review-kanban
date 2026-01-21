use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// Webhook 审计状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookAuditStatus {
    Processed,
    SignatureInvalid,
    RepoUnmapped,
    WebhookDuplicate,
    WebhookIgnored,
    Error,
}

impl std::fmt::Display for WebhookAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookAuditStatus::Processed => write!(f, "processed"),
            WebhookAuditStatus::SignatureInvalid => write!(f, "signature_invalid"),
            WebhookAuditStatus::RepoUnmapped => write!(f, "repo_unmapped"),
            WebhookAuditStatus::WebhookDuplicate => write!(f, "webhook_duplicate"),
            WebhookAuditStatus::WebhookIgnored => write!(f, "webhook_ignored"),
            WebhookAuditStatus::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for WebhookAuditStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "processed" => Ok(WebhookAuditStatus::Processed),
            "signature_invalid" => Ok(WebhookAuditStatus::SignatureInvalid),
            "repo_unmapped" => Ok(WebhookAuditStatus::RepoUnmapped),
            "webhook_duplicate" => Ok(WebhookAuditStatus::WebhookDuplicate),
            "webhook_ignored" => Ok(WebhookAuditStatus::WebhookIgnored),
            "error" => Ok(WebhookAuditStatus::Error),
            _ => Err(format!("未知的 Webhook 审计状态: {}", s)),
        }
    }
}

/// Webhook 审计记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookAudit {
    pub id: String,
    pub gitea_repo: String,
    pub event_type: String,
    pub payload_hash: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl WebhookAudit {
    /// 创建审计记录
    pub async fn create(
        pool: &Pool<Sqlite>,
        gitea_repo: &str,
        event_type: &str,
        payload_hash: &str,
        status: WebhookAuditStatus,
        error_message: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO webhook_audits (id, gitea_repo, event_type, payload_hash, status, error_message, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(gitea_repo)
        .bind(event_type)
        .bind(payload_hash)
        .bind(status.to_string())
        .bind(error_message)
        .bind(now)
        .execute(pool)
        .await?;

        sqlx::query_as::<_, Self>("SELECT * FROM webhook_audits WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await
    }

    /// 检查是否重复
    pub async fn is_duplicate(pool: &Pool<Sqlite>, payload_hash: &str) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM webhook_audits WHERE payload_hash = ?"
        )
        .bind(payload_hash)
        .fetch_one(pool)
        .await?;

        Ok(count.0 > 0)
    }

    /// 列出审计记录
    pub async fn list(
        pool: &Pool<Sqlite>,
        limit: i64,
        statuses: &[WebhookAuditStatus],
    ) -> Result<Vec<Self>, sqlx::Error> {
        if statuses.is_empty() {
            return sqlx::query_as::<_, Self>(
                "SELECT * FROM webhook_audits ORDER BY created_at DESC LIMIT ?"
            )
            .bind(limit)
            .fetch_all(pool)
            .await;
        }

        let status_strs: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
        let placeholders = status_strs.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let query = format!(
            "SELECT * FROM webhook_audits WHERE status IN ({}) ORDER BY created_at DESC LIMIT ?",
            placeholders
        );

        let mut q = sqlx::query_as::<_, Self>(&query);
        for status in &status_strs {
            q = q.bind(status);
        }
        q = q.bind(limit);

        q.fetch_all(pool).await
    }
}
