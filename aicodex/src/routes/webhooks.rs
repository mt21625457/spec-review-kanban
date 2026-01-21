use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Sha256, Digest};

use crate::db::models::repo_mapping::RepoMapping;
use crate::db::models::webhook_audit::{WebhookAudit, WebhookAuditStatus};
use crate::error::ApiError;
use crate::services::webhook_handler::WebhookEventType;
use crate::AppState;

/// 处理 Gitea Webhook
async fn handle_gitea_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let result = state.services.webhook.handle_gitea_webhook(&headers, &body).await?;

    // 检查是否需要处理
    match result.audit_status {
        WebhookAuditStatus::SignatureInvalid => {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": "签名验证失败",
                })),
            ));
        }
        WebhookAuditStatus::WebhookIgnored | WebhookAuditStatus::WebhookDuplicate => {
            return Ok((
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": "Webhook 已忽略",
                    "reason": result.error,
                })),
            ));
        }
        _ => {}
    }

    // 检查事件类型
    if !matches!(
        result.event_type,
        WebhookEventType::PullRequestOpened | WebhookEventType::PullRequestSynchronized
    ) {
        return Ok((
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": "事件类型已忽略",
            })),
        ));
    }

    // 获取 payload
    let payload = result.payload.ok_or_else(|| {
        ApiError::BadRequest("无法解析 Webhook payload".to_string())
    })?;

    // 检查仓库映射
    let repo_name = &payload.repository.full_name;
    let mapping = RepoMapping::get_by_gitea_repo(&state.db.pool, repo_name).await?;

    if mapping.is_none() {
        // 记录未映射的审计日志
        WebhookAudit::create(
            &state.db.pool,
            repo_name,
            "pull_request",
            &hex::encode(Sha256::digest(&body)),
            WebhookAuditStatus::RepoUnmapped,
            Some(&format!("仓库 {} 未配置映射", repo_name)),
        )
        .await?;

        return Ok((
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("仓库 {} 未配置映射", repo_name),
            })),
        ));
    }

    // 处理审核请求
    let review_run = state.services.review.handle_pr_review(&payload).await?;

    // 记录成功的审计日志
    WebhookAudit::create(
        &state.db.pool,
        repo_name,
        "pull_request",
        &format!("{:x}", sha2::Sha256::digest(&body) as sha2::digest::Output<sha2::Sha256>),
        WebhookAuditStatus::Processed,
        None,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": {
                "review_run_id": review_run.id,
                "status": review_run.status,
            },
        })),
    ))
}

#[derive(Debug, Deserialize)]
struct AuditQuery {
    limit: Option<i64>,
    status: Option<String>,
}

/// 列出 Webhook 审计日志
async fn list_audits(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(200);

    // 解析状态过滤
    let statuses: Vec<WebhookAuditStatus> = query
        .status
        .map(|s| {
            s.split(',')
                .filter_map(|item| match item.trim() {
                    "processed" => Some(WebhookAuditStatus::Processed),
                    "signature_invalid" => Some(WebhookAuditStatus::SignatureInvalid),
                    "repo_unmapped" => Some(WebhookAuditStatus::RepoUnmapped),
                    "webhook_duplicate" => Some(WebhookAuditStatus::WebhookDuplicate),
                    "webhook_ignored" => Some(WebhookAuditStatus::WebhookIgnored),
                    "error" => Some(WebhookAuditStatus::Error),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    let audits = WebhookAudit::list(&state.db.pool, limit, &statuses).await?;

    Ok(Json(json!({
        "success": true,
        "data": audits,
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/gitea", post(handle_gitea_webhook))
        .route("/audits", get(list_audits))
}
