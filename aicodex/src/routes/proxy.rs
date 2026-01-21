//! 代理路由
//!
//! 将请求代理到用户当前选中的 vibe-kanban 实例

use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Json, Router,
};
use serde::Serialize;

use crate::db::models::vibe_instance::InstanceStatus;
use crate::AppState;

/// 代理错误响应
#[derive(Debug, Serialize)]
struct ProxyErrorResponse {
    success: bool,
    error: String,
    code: String,
}

impl ProxyErrorResponse {
    fn unauthorized(message: &str) -> Self {
        Self {
            success: false,
            error: message.to_string(),
            code: "UNAUTHORIZED".to_string(),
        }
    }

    fn no_instance(message: &str) -> Self {
        Self {
            success: false,
            error: message.to_string(),
            code: "NO_INSTANCE".to_string(),
        }
    }

    fn instance_not_running(message: &str) -> Self {
        Self {
            success: false,
            error: message.to_string(),
            code: "INSTANCE_NOT_RUNNING".to_string(),
        }
    }

    fn proxy_error(message: &str) -> Self {
        Self {
            success: false,
            error: message.to_string(),
            code: "PROXY_ERROR".to_string(),
        }
    }
}

/// 从请求中提取认证令牌
fn extract_token(req: &Request<Body>) -> Option<String> {
    // 优先从 Authorization header 提取
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
    }

    // 从 Cookie 提取
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookies) = cookie_header.to_str() {
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("auth_token=") {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

/// 代理请求到用户当前实例
async fn proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    // 1. 提取认证令牌
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ProxyErrorResponse::unauthorized("未提供认证令牌")),
            )
                .into_response();
        }
    };

    // 2. 验证会话并获取用户
    let user = match state.services.user_manager.verify_session(&token).await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ProxyErrorResponse::unauthorized(&format!("认证失败: {}", e))),
            )
                .into_response();
        }
    };

    // 3. 获取用户当前实例 ID
    let instance_id = match &user.current_instance_id {
        Some(id) => id.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ProxyErrorResponse::no_instance("未选择工作实例，请先切换到一个实例")),
            )
                .into_response();
        }
    };

    // 4. 验证用户有权访问该实例
    let is_assigned = match state
        .services
        .user_manager
        .is_user_assigned_to_instance(&user.id, &instance_id)
        .await
    {
        Ok(assigned) => assigned,
        Err(e) => {
            tracing::error!(error = %e, "检查用户实例分配失败");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProxyErrorResponse::proxy_error("检查实例权限失败")),
            )
                .into_response();
        }
    };

    if !is_assigned {
        return (
            StatusCode::FORBIDDEN,
            Json(ProxyErrorResponse::unauthorized("无权访问此实例")),
        )
            .into_response();
    }

    // 5. 获取实例信息
    let instance = match state.services.instance_manager.get(&instance_id).await {
        Ok(Some(i)) => i,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ProxyErrorResponse::no_instance("当前实例不存在")),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "获取实例信息失败");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProxyErrorResponse::proxy_error("获取实例信息失败")),
            )
                .into_response();
        }
    };

    // 6. 检查实例状态，如果未运行且配置了自动启动，则启动
    if instance.status_enum() != InstanceStatus::Running {
        if instance.auto_start {
            tracing::info!(instance_id = %instance_id, "自动启动实例");
            if let Err(e) = state.services.instance_manager.start(&instance_id).await {
                tracing::warn!(instance_id = %instance_id, error = %e, "自动启动实例失败");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ProxyErrorResponse::instance_not_running(&format!(
                        "实例启动失败: {}",
                        e
                    ))),
                )
                    .into_response();
            }
        } else {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ProxyErrorResponse::instance_not_running("实例未运行且未配置自动启动")),
            )
                .into_response();
        }
    }

    // 7. 构建目标 URL
    let method = req.method().clone();
    let vibe_path = format!("/api/{}", path);
    let target_path = if let Some(query) = req.uri().query() {
        format!("{}?{}", vibe_path, query)
    } else {
        vibe_path
    };

    let target_url = format!("http://127.0.0.1:{}{}", instance.port, target_path);

    tracing::debug!(
        instance_id = %instance_id,
        target_url = %target_url,
        method = %method,
        "代理请求"
    );

    // 8. 读取请求体
    let body = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => {
            if bytes.is_empty() {
                None
            } else {
                Some(bytes.to_vec())
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "读取请求体失败");
            return (
                StatusCode::BAD_REQUEST,
                Json(ProxyErrorResponse::proxy_error("读取请求体失败")),
            )
                .into_response();
        }
    };

    // 9. 构建并发送代理请求
    let client = reqwest::Client::new();
    let mut request_builder = client.request(method.clone(), &target_url);

    if let Some(body_bytes) = body {
        request_builder = request_builder.body(body_bytes);
        request_builder = request_builder.header("Content-Type", "application/json");
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = StatusCode::from_u16(response.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            let content_type = response
                .headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();

            match response.bytes().await {
                Ok(body) => Response::builder()
                    .status(status)
                    .header("Content-Type", content_type)
                    .body(Body::from(body))
                    .unwrap_or_else(|_| {
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from("响应构建失败"))
                            .unwrap()
                    }),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    Json(ProxyErrorResponse::proxy_error(&format!("读取响应失败: {}", e))),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(ProxyErrorResponse::proxy_error(&format!("代理请求失败: {}", e))),
        )
            .into_response(),
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/{*path}", any(proxy_handler))
}
