//! 用户认证 API
//!
//! 提供用户注册、登录、登出等认证功能

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::middleware::AuthenticatedUser;
use crate::AppState;

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 修改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 用户信息响应
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: String,
    pub current_instance_id: Option<String>,
    pub is_active: bool,
}

/// 创建认证路由
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/password", put(change_password))
}

/// 用户注册
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 验证输入
    if req.username.len() < 3 {
        return Err(ApiError::BadRequest("用户名至少需要 3 个字符".to_string()));
    }
    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("密码至少需要 6 个字符".to_string()));
    }

    let user = state
        .services
        .user_manager
        .register(
            &req.username,
            req.email.as_deref(),
            &req.password,
            req.display_name.as_deref(),
            None,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "user": {
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "display_name": user.display_name,
                "role": user.role,
            }
        })),
    ))
}

/// 用户登录
async fn login(
    State(state): State<AppState>,
    cookies: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state
        .services
        .user_manager
        .login(&req.username, &req.password, None, None)
        .await?;

    // 设置 Cookie
    let cookie = axum_extra::extract::cookie::Cookie::build(("auth_token", response.token.clone()))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::days(1))
        .build();

    let jar = cookies.add(cookie);

    Ok((
        jar,
        Json(serde_json::json!({
            "success": true,
            "data": {
                "token": response.token,
                "user": response.user,
                "instances": response.instances,
                "current_instance_id": response.current_instance_id,
            }
        })),
    ))
}

/// 用户登出
async fn logout(
    State(state): State<AppState>,
    cookies: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    // 从 Cookie 获取 token
    if let Some(cookie) = cookies.get("auth_token") {
        let _ = state.services.user_manager.logout(cookie.value()).await;
    }

    // 删除 Cookie
    let cookie = axum_extra::extract::cookie::Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::ZERO)
        .build();

    let jar = cookies.remove(cookie);

    Ok((
        jar,
        Json(serde_json::json!({
            "success": true,
            "message": "已登出"
        })),
    ))
}

/// 获取当前用户
async fn get_current_user(
    State(state): State<AppState>,
    cookies: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let token = cookies
        .get("auth_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("未登录".to_string()))?;

    let (user, instances) = state.services.user_manager.get_current_user(&token).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "user": {
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "display_name": user.display_name,
                "role": user.role,
                "current_instance_id": user.current_instance_id,
                "is_active": user.is_active,
            },
            "instances": instances,
        }
    })))
}

/// 修改密码
async fn change_password(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.new_password.len() < 6 {
        return Err(ApiError::BadRequest("新密码至少需要 6 个字符".to_string()));
    }

    state
        .services
        .user_manager
        .change_password(auth_user.id(), &req.old_password, &req.new_password)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "密码已修改"
    })))
}
