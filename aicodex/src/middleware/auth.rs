//! 认证中间件和 Extractor
//!
//! 提供请求认证和权限验证功能

use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{header, request::Parts, Request, StatusCode},
    Json,
};
use serde::Serialize;

use crate::db::models::user::User;
use crate::AppState;

/// 认证错误响应
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub code: String,
}

impl AuthErrorResponse {
    pub fn unauthorized(message: &str) -> Self {
        Self {
            error: message.to_string(),
            code: "UNAUTHORIZED".to_string(),
        }
    }

    pub fn forbidden(message: &str) -> Self {
        Self {
            error: message.to_string(),
            code: "FORBIDDEN".to_string(),
        }
    }
}

/// 已认证用户 - 存储在请求扩展中
#[derive(Clone)]
pub struct AuthenticatedUser(pub User);

impl AuthenticatedUser {
    /// 获取用户引用
    pub fn user(&self) -> &User {
        &self.0
    }

    /// 获取用户 ID
    pub fn id(&self) -> &str {
        &self.0.id
    }

    /// 获取用户名
    pub fn username(&self) -> &str {
        &self.0.username
    }

    /// 是否为管理员
    pub fn is_admin(&self) -> bool {
        self.0.is_admin()
    }

    /// 获取当前实例 ID
    pub fn current_instance_id(&self) -> Option<&str> {
        self.0.current_instance_id.as_deref()
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, Json<AuthErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 尝试从 Authorization header 提取
        let token = if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                auth_str.strip_prefix("Bearer ").map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // 如果没有 header，尝试从 Cookie 提取
        let token = if let Some(t) = token {
            t
        } else {
            // 从 Cookie 提取
            let cookies = parts
                .headers
                .get(header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            // 解析 cookie
            let mut auth_token = None;
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("auth_token=") {
                    auth_token = Some(value.to_string());
                    break;
                }
            }

            auth_token.ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse::unauthorized("未提供认证令牌")),
                )
            })?
        };

        // 验证会话
        let user = state
            .services
            .user_manager
            .verify_session(&token)
            .await
            .map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse::unauthorized(&e.to_string())),
                )
            })?;

        Ok(AuthenticatedUser(user))
    }
}

/// 管理员用户 Extractor
#[derive(Clone)]
pub struct AdminUser(pub User);

impl AdminUser {
    pub fn user(&self) -> &User {
        &self.0
    }
}

impl std::ops::Deref for AdminUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = (StatusCode, Json<AuthErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user: AuthenticatedUser =
            AuthenticatedUser::from_request_parts(parts, state).await?;

        if !auth_user.is_admin() {
            return Err((
                StatusCode::FORBIDDEN,
                Json(AuthErrorResponse::forbidden("需要管理员权限")),
            ));
        }

        Ok(AdminUser(auth_user.0))
    }
}

/// 从请求扩展中提取已认证用户
pub fn get_authenticated_user(req: &Request<Body>) -> Option<&AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>()
}
