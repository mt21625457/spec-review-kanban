use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::AppState;

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i64>,
}

/// 列出审核运行
async fn list_reviews(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(200);
    let reviews = state.services.review.list_reviews(limit).await?;

    Ok(Json(json!({
        "success": true,
        "data": reviews,
    })))
}

/// 获取审核详情
async fn get_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let review = state
        .services
        .review
        .get_review(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("审核运行 {} 不存在", id)))?;

    let events = state.services.review.get_review_events(&id).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "review": review,
            "events": events,
        },
    })))
}

/// 重新运行审核
async fn rerun_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let review = state.services.review.rerun_review(&id).await?;

    Ok(Json(json!({
        "success": true,
        "data": review,
    })))
}

/// 取消审核
async fn cancel_review(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let review = state.services.review.cancel_review(&id).await?;

    Ok(Json(json!({
        "success": true,
        "data": review,
    })))
}

/// 获取审核事件
async fn get_review_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let events = state.services.review.get_review_events(&id).await?;

    Ok(Json(json!({
        "success": true,
        "data": events,
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_reviews))
        .route("/{id}", get(get_review))
        .route("/{id}/rerun", post(rerun_review))
        .route("/{id}/cancel", post(cancel_review))
        .route("/{id}/events", get(get_review_events))
}
