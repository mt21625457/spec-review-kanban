use axum::{
    extract::State,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde_json::json;

use crate::error::ApiError;
use crate::services::config_service::{AllSettings, GiteaSettings, QueueSettings, ReviewSettings};
use crate::AppState;

/// 获取所有配置
async fn get_all_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let settings = state.services.config.get_all_settings().await?;
    Ok(Json(json!({
        "success": true,
        "data": settings,
    })))
}

/// 更新 Gitea 配置
async fn update_gitea_settings(
    State(state): State<AppState>,
    Json(settings): Json<GiteaSettings>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.services.config.update_gitea_settings(&settings).await?;
    let updated = state.services.config.get_gitea_settings().await?;
    Ok(Json(json!({
        "success": true,
        "data": updated,
    })))
}

/// 测试 Gitea 连接
async fn test_gitea_connection(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = state.services.gitea.test_connection().await;
    Ok(Json(json!({
        "success": result.success,
        "data": result,
    })))
}

/// 更新审核配置
async fn update_review_settings(
    State(state): State<AppState>,
    Json(settings): Json<ReviewSettings>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.services.config.update_review_settings(&settings).await?;
    let updated = state.services.config.get_review_settings().await?;
    Ok(Json(json!({
        "success": true,
        "data": updated,
    })))
}

/// 更新队列配置
async fn update_queue_settings(
    State(state): State<AppState>,
    Json(settings): Json<QueueSettings>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.services.config.update_queue_settings(&settings).await?;
    let updated = state.services.config.get_queue_settings().await?;
    Ok(Json(json!({
        "success": true,
        "data": updated,
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_settings))
        .route("/gitea", put(update_gitea_settings))
        .route("/gitea/test", post(test_gitea_connection))
        .route("/review", put(update_review_settings))
        .route("/queue", put(update_queue_settings))
}
