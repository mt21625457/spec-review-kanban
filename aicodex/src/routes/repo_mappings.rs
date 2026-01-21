use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;

use crate::db::models::repo_mapping::{CreateRepoMapping, RepoMapping, UpdateRepoMapping};
use crate::error::ApiError;
use crate::AppState;

/// 列出所有仓库映射
async fn list_repo_mappings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mappings = RepoMapping::list(&state.db.pool).await?;
    Ok(Json(json!({
        "success": true,
        "data": mappings,
    })))
}

/// 获取单个仓库映射
async fn get_repo_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mapping = RepoMapping::get_by_id(&state.db.pool, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("仓库映射 {} 不存在", id)))?;
    Ok(Json(json!({
        "success": true,
        "data": mapping,
    })))
}

/// 创建仓库映射
async fn create_repo_mapping(
    State(state): State<AppState>,
    Json(input): Json<CreateRepoMapping>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // 检查是否已存在
    if RepoMapping::get_by_gitea_repo(&state.db.pool, &input.gitea_repo)
        .await?
        .is_some()
    {
        return Err(ApiError::BadRequest(format!(
            "仓库 {} 已存在映射",
            input.gitea_repo
        )));
    }

    let mapping = RepoMapping::create(&state.db.pool, input).await?;
    Ok(Json(json!({
        "success": true,
        "data": mapping,
    })))
}

/// 更新仓库映射
async fn update_repo_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateRepoMapping>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mapping = RepoMapping::update(&state.db.pool, &id, input).await?;
    Ok(Json(json!({
        "success": true,
        "data": mapping,
    })))
}

/// 删除仓库映射
async fn delete_repo_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let deleted = RepoMapping::delete(&state.db.pool, &id).await?;
    if !deleted {
        return Err(ApiError::NotFound(format!("仓库映射 {} 不存在", id)));
    }
    Ok(Json(json!({
        "success": true,
        "message": "删除成功",
    })))
}

/// 同步仓库到 Vibe Kanban
async fn sync_repo_mapping(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mapping = RepoMapping::get_by_id(&state.db.pool, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("仓库映射 {} 不存在", id)))?;

    // TODO: 实现与 Vibe Kanban 的同步逻辑
    // 例如：创建或更新 Vibe Kanban 中的项目

    Ok(Json(json!({
        "success": true,
        "data": mapping,
        "message": "同步功能待实现",
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_repo_mappings))
        .route("/", post(create_repo_mapping))
        .route("/{id}", get(get_repo_mapping))
        .route("/{id}", put(update_repo_mapping))
        .route("/{id}", delete(delete_repo_mapping))
        .route("/{id}/sync", post(sync_repo_mapping))
}
