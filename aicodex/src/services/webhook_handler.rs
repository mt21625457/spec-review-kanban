use std::sync::Arc;

use axum::http::HeaderMap;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

use crate::db::models::webhook_audit::{WebhookAudit, WebhookAuditStatus};
use crate::db::Database;

/// Webhook 事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum WebhookEventType {
    PullRequestOpened,
    PullRequestSynchronized,
    PullRequestClosed,
    Unknown(String),
}

/// PR Webhook 载荷
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub number: i64,
    pub pull_request: PullRequestInfo,
    pub repository: RepositoryInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestInfo {
    pub number: i64,
    pub title: String,
    pub html_url: String,
    pub head: BranchInfo,
    pub base: BranchInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BranchInfo {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryInfo {
    pub full_name: String,
    pub clone_url: String,
}

/// Webhook 处理结果
pub struct WebhookResult {
    pub event_type: WebhookEventType,
    pub payload: Option<PullRequestPayload>,
    pub audit_status: WebhookAuditStatus,
    pub error: Option<String>,
}

/// Webhook 处理器
pub struct WebhookHandler {
    db: Arc<Database>,
    secret: Option<String>,
}

impl WebhookHandler {
    pub fn new(db: Arc<Database>, secret: Option<String>) -> Self {
        Self { db, secret }
    }

    /// 处理 Gitea Webhook
    pub async fn handle_gitea_webhook(
        &self,
        headers: &HeaderMap,
        body: &[u8],
    ) -> anyhow::Result<WebhookResult> {
        // 获取事件类型
        let event_header = headers
            .get("X-Gitea-Event")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        // 验证签名
        if let Some(ref secret) = self.secret {
            if !self.verify_signature(headers, body, secret)? {
                // 尝试解析仓库名用于审计
                let repo_name = self.extract_repo_name(body);
                WebhookAudit::create(
                    &self.db.pool,
                    &repo_name,
                    event_header,
                    &self.compute_payload_hash(body),
                    WebhookAuditStatus::SignatureInvalid,
                    Some("签名验证失败"),
                )
                .await?;

                return Ok(WebhookResult {
                    event_type: WebhookEventType::Unknown(event_header.to_string()),
                    payload: None,
                    audit_status: WebhookAuditStatus::SignatureInvalid,
                    error: Some("签名验证失败".to_string()),
                });
            }
        }

        // 只处理 pull_request 事件
        if event_header != "pull_request" {
            let repo_name = self.extract_repo_name(body);
            WebhookAudit::create(
                &self.db.pool,
                &repo_name,
                event_header,
                &self.compute_payload_hash(body),
                WebhookAuditStatus::WebhookIgnored,
                Some(&format!("忽略事件类型: {}", event_header)),
            )
            .await?;

            return Ok(WebhookResult {
                event_type: WebhookEventType::Unknown(event_header.to_string()),
                payload: None,
                audit_status: WebhookAuditStatus::WebhookIgnored,
                error: None,
            });
        }

        // 解析载荷
        let payload: PullRequestPayload = serde_json::from_slice(body)?;
        let repo_name = payload.repository.full_name.clone();
        let payload_hash = self.compute_payload_hash(body);

        // 检查重复
        if WebhookAudit::is_duplicate(&self.db.pool, &payload_hash).await? {
            WebhookAudit::create(
                &self.db.pool,
                &repo_name,
                event_header,
                &payload_hash,
                WebhookAuditStatus::WebhookDuplicate,
                Some("重复的 Webhook"),
            )
            .await?;

            return Ok(WebhookResult {
                event_type: WebhookEventType::Unknown(event_header.to_string()),
                payload: Some(payload),
                audit_status: WebhookAuditStatus::WebhookDuplicate,
                error: None,
            });
        }

        // 确定事件类型
        let event_type = match payload.action.as_str() {
            "opened" => WebhookEventType::PullRequestOpened,
            "synchronize" => WebhookEventType::PullRequestSynchronized,
            "closed" => WebhookEventType::PullRequestClosed,
            other => WebhookEventType::Unknown(other.to_string()),
        };

        // 只处理 opened 和 synchronize
        if !matches!(
            event_type,
            WebhookEventType::PullRequestOpened | WebhookEventType::PullRequestSynchronized
        ) {
            WebhookAudit::create(
                &self.db.pool,
                &repo_name,
                event_header,
                &payload_hash,
                WebhookAuditStatus::WebhookIgnored,
                Some(&format!("忽略 PR 动作: {}", payload.action)),
            )
            .await?;

            return Ok(WebhookResult {
                event_type,
                payload: Some(payload),
                audit_status: WebhookAuditStatus::WebhookIgnored,
                error: None,
            });
        }

        Ok(WebhookResult {
            event_type,
            payload: Some(payload),
            audit_status: WebhookAuditStatus::Processed,
            error: None,
        })
    }

    /// 验证 HMAC 签名
    fn verify_signature(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        secret: &str,
    ) -> anyhow::Result<bool> {
        let signature = headers
            .get("X-Gitea-Signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("缺少签名头"))?;

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
        mac.update(body);

        let expected = hex::encode(mac.finalize().into_bytes());

        Ok(signature == expected)
    }

    /// 计算载荷哈希
    fn compute_payload_hash(&self, body: &[u8]) -> String {
        use sha2::Digest;
        let hash = sha2::Sha256::digest(body);
        hex::encode(hash)
    }

    /// 从载荷中提取仓库名
    fn extract_repo_name(&self, body: &[u8]) -> String {
        #[derive(Deserialize)]
        struct MinimalPayload {
            repository: Option<MinimalRepo>,
        }
        #[derive(Deserialize)]
        struct MinimalRepo {
            full_name: Option<String>,
        }

        serde_json::from_slice::<MinimalPayload>(body)
            .ok()
            .and_then(|p| p.repository)
            .and_then(|r| r.full_name)
            .unwrap_or_else(|| "unknown".to_string())
    }
}
