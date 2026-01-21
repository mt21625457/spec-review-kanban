//! 实例管理 API（管理员）
//!
//! 提供 vibe-kanban 实例的 CRUD 和生命周期管理

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::AuthenticatedUser;
use crate::services::agent_config_manager::AgentConfigRequest;
use crate::AppState;

/// 创建实例请求
#[derive(Debug, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    pub max_users: Option<i32>,
}

fn default_auto_start() -> bool {
    true
}

/// 更新实例请求
#[derive(Debug, Deserialize)]
pub struct UpdateInstanceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub auto_start: Option<bool>,
    pub max_users: Option<i32>,
}

/// 创建实例管理路由
pub fn instances_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_instances).post(create_instance))
        .route("/{id}", get(get_instance).put(update_instance).delete(delete_instance))
        .route("/{id}/start", post(start_instance))
        .route("/{id}/stop", post(stop_instance))
        .route("/{id}/restart", post(restart_instance))
        .route("/{id}/health", get(health_check))
        .route("/{id}/users", get(get_instance_users))
        .route("/{id}/agents", get(list_agents))
        .route("/{id}/agents/{agent_type}", put(set_agent_config))
        .route("/{id}/agents/{agent_type}/test", post(test_agent_connection))
}

/// 列出所有实例
async fn list_instances(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let instances = state.services.instance_manager.list().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instances": instances,
        }
    })))
}

/// 创建实例
async fn create_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(req): Json<CreateInstanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("实例名称不能为空".to_string()));
    }

    let instance = state
        .services
        .instance_manager
        .create(
            &req.name,
            req.description.as_deref(),
            req.auto_start,
            req.max_users,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "instance": crate::db::models::vibe_instance::InstanceInfo::from(instance),
            }
        })),
    ))
}

/// 获取实例详情
async fn get_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let instance = state
        .services
        .instance_manager
        .get(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("实例不存在".to_string()))?;

    let users = state.services.instance_manager.get_instance_users(&id).await?;
    let agents = state.services.agent_config_manager.list_configs(&id).await?;

    let mut info = crate::db::models::vibe_instance::InstanceInfo::from(instance);
    info.user_count = Some(users.len() as i64);

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instance": info,
            "users": users,
            "agents": agents,
        }
    })))
}

/// 更新实例
async fn update_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateInstanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let instance = state
        .services
        .instance_manager
        .update(
            &id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.auto_start,
            req.max_users,
        )
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instance": crate::db::models::vibe_instance::InstanceInfo::from(instance),
        }
    })))
}

/// 删除实例
async fn delete_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    state.services.instance_manager.delete(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "实例已删除",
    })))
}

/// 启动实例
async fn start_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    state.services.instance_manager.start(&id).await?;

    let instance = state
        .services
        .instance_manager
        .get(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("实例不存在".to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "实例已启动",
        "data": {
            "instance": crate::db::models::vibe_instance::InstanceInfo::from(instance),
        }
    })))
}

/// 停止实例
async fn stop_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    state.services.instance_manager.stop(&id).await?;

    let instance = state
        .services
        .instance_manager
        .get(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("实例不存在".to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "实例已停止",
        "data": {
            "instance": crate::db::models::vibe_instance::InstanceInfo::from(instance),
        }
    })))
}

/// 重启实例
async fn restart_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    state.services.instance_manager.restart(&id).await?;

    let instance = state
        .services
        .instance_manager
        .get(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("实例不存在".to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "实例已重启",
        "data": {
            "instance": crate::db::models::vibe_instance::InstanceInfo::from(instance),
        }
    })))
}

/// 健康检查
async fn health_check(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let health = state.services.instance_manager.health_check(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "health_status": health.to_string(),
        }
    })))
}

/// 获取实例用户
async fn get_instance_users(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let users = state.services.instance_manager.get_instance_users(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "users": users,
        }
    })))
}

/// 列出智能体配置
async fn list_agents(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let agents = state.services.agent_config_manager.list_configs(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "agents": agents,
        }
    })))
}

/// 设置智能体配置
async fn set_agent_config(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((id, agent_type)): Path<(String, String)>,
    Json(req): Json<AgentConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let config = state
        .services
        .agent_config_manager
        .set_config(&id, &agent_type, req)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "agent": config,
        }
    })))
}

/// 测试智能体连接
async fn test_agent_connection(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((id, agent_type)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let success = state
        .services
        .agent_config_manager
        .test_connection(&id, &agent_type)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "connection_ok": success,
        }
    })))
}
