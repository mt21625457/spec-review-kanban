//! 用户管理 API（管理员）
//!
//! 提供用户 CRUD 和实例分配功能

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, put},
    Json, Router,
};
use serde::Deserialize;

use crate::db::models::user::UserRole;
use crate::error::ApiError;
use crate::middleware::AuthenticatedUser;
use crate::AppState;

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
}

/// 分配实例请求
#[derive(Debug, Deserialize)]
pub struct AssignInstanceRequest {
    pub instance_ids: Vec<String>,
}

/// 激活/停用请求
#[derive(Debug, Deserialize)]
pub struct SetActiveRequest {
    pub is_active: bool,
}

/// 重置密码请求
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

/// 创建用户管理路由
pub fn users_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/{id}/instances", get(get_user_instances).post(assign_instances))
        .route("/{id}/instances/{instance_id}", delete(unassign_instance))
        .route("/{id}/activate", put(set_active))
        .route("/{id}/password", put(reset_password))
}

/// 列出所有用户
async fn list_users(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    let users = state.services.user_manager.list_users().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "users": users,
        }
    })))
}

/// 创建用户
async fn create_user(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    if req.username.len() < 3 {
        return Err(ApiError::BadRequest("用户名至少需要 3 个字符".to_string()));
    }
    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("密码至少需要 6 个字符".to_string()));
    }

    let role = req
        .role
        .as_ref()
        .and_then(|r| r.parse::<UserRole>().ok())
        .unwrap_or(UserRole::User);

    let user = state
        .services
        .user_manager
        .register(
            &req.username,
            req.email.as_deref(),
            &req.password,
            req.display_name.as_deref(),
            Some(role),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "user": crate::db::models::user::UserInfo::from(user),
            }
        })),
    ))
}

/// 获取用户详情
async fn get_user(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // 允许用户查看自己的信息，或管理员查看任何人
    if !auth_user.is_admin() && auth_user.id() != id {
        return Err(ApiError::Forbidden("无权查看此用户".to_string()));
    }

    let user = state
        .services
        .user_manager
        .get_user(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound("用户不存在".to_string()))?;

    let instances = state.services.user_manager.get_user_instances(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "user": user,
            "instances": instances,
        }
    })))
}

/// 更新用户信息
async fn update_user(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 允许用户更新自己的基本信息，但角色只能管理员修改
    if !auth_user.is_admin() && auth_user.id() != id {
        return Err(ApiError::Forbidden("无权修改此用户".to_string()));
    }

    // 非管理员不能修改角色
    let role = if auth_user.is_admin() {
        req.role.as_ref().and_then(|r| r.parse::<UserRole>().ok())
    } else {
        None
    };

    let user = state
        .services
        .user_manager
        .update_user(&id, req.email.as_deref(), req.display_name.as_deref(), role)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "user": user,
        }
    })))
}

/// 删除用户
async fn delete_user(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    // 不能删除自己
    if auth_user.id() == id {
        return Err(ApiError::BadRequest("不能删除自己".to_string()));
    }

    state.services.user_manager.delete_user(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "用户已删除",
    })))
}

/// 获取用户分配的实例
async fn get_user_instances(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() && auth_user.id() != id {
        return Err(ApiError::Forbidden("无权查看".to_string()));
    }

    let instances = state.services.user_manager.get_user_instances(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instances": instances,
        }
    })))
}

/// 分配用户到实例
async fn assign_instances(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(req): Json<AssignInstanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    for instance_id in &req.instance_ids {
        state
            .services
            .user_manager
            .assign_user_to_instance(auth_user.user(), &id, instance_id)
            .await?;
    }

    let instances = state.services.user_manager.get_user_instances(&id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "用户已分配到实例",
        "instances": instances,
    })))
}

/// 取消用户的实例分配
async fn unassign_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path((user_id, instance_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    state
        .services
        .user_manager
        .unassign_user_from_instance(auth_user.user(), &user_id, &instance_id)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已取消用户的实例分配",
    })))
}

/// 激活/停用用户
async fn set_active(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(req): Json<SetActiveRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    // 不能停用自己
    if auth_user.id() == id && !req.is_active {
        return Err(ApiError::BadRequest("不能停用自己".to_string()));
    }

    state
        .services
        .user_manager
        .set_user_active(&id, req.is_active)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": if req.is_active { "用户已激活" } else { "用户已停用" },
    })))
}

/// 重置密码
async fn reset_password(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if !auth_user.is_admin() {
        return Err(ApiError::Forbidden("需要管理员权限".to_string()));
    }

    if req.new_password.len() < 6 {
        return Err(ApiError::BadRequest("密码至少需要 6 个字符".to_string()));
    }

    state
        .services
        .user_manager
        .reset_password(&id, &req.new_password)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "密码已重置",
    })))
}
