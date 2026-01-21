//! HTTP 请求追踪中间件模块

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{Instrument, Span};
use uuid::Uuid;

/// X-Request-Id 响应头名称
pub const X_REQUEST_ID: &str = "X-Request-Id";
/// X-Trace-Id 请求头名称
pub const X_TRACE_ID: &str = "X-Trace-Id";
/// W3C Trace Context 请求头名称
pub const TRACEPARENT: &str = "traceparent";

/// 请求追踪中间件
///
/// 为每个请求生成唯一的 request_id，并在响应头中返回。
/// 支持从请求头提取外部 trace_id。
pub async fn request_tracing_middleware(request: Request, next: Next) -> Response {
    // 生成或提取 request_id
    let request_id = extract_or_generate_request_id(&request);

    // 提取外部 trace_id（用于分布式追踪）
    let trace_id = extract_trace_id(&request);

    // 提取请求信息
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // 提取客户端 IP（简化版，实际可能需要考虑代理头）
    let client_ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-IP")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    // 提取 User-Agent
    let user_agent = request
        .headers()
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 创建请求 span
    let span = if let Some(trace_id) = &trace_id {
        tracing::info_span!(
            "http_request",
            request_id = %request_id,
            trace_id = %trace_id,
            method = %method,
            path = %path,
            client_ip = ?client_ip,
        )
    } else {
        tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %method,
            path = %path,
            client_ip = ?client_ip,
        )
    };

    // 记录请求开始
    span.in_scope(|| {
        tracing::info!(
            user_agent = ?user_agent,
            "收到请求"
        );
    });

    let start = std::time::Instant::now();

    // 执行请求
    let response = next.run(request).instrument(span.clone()).await;

    // 计算耗时
    let duration = start.elapsed();
    let status = response.status();

    // 记录请求完成
    span.in_scope(|| {
        let level = if status.is_server_error() {
            tracing::Level::ERROR
        } else if status.is_client_error() {
            tracing::Level::WARN
        } else {
            tracing::Level::INFO
        };

        match level {
            tracing::Level::ERROR => {
                tracing::error!(
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "请求完成"
                );
            }
            tracing::Level::WARN => {
                tracing::warn!(
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "请求完成"
                );
            }
            _ => {
                tracing::info!(
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "请求完成"
                );
            }
        }
    });

    // 在响应头中添加 request_id
    let mut response = response;
    response
        .headers_mut()
        .insert(X_REQUEST_ID, request_id.parse().unwrap());

    response
}

/// 提取或生成 request_id
fn extract_or_generate_request_id(request: &Request) -> String {
    // 尝试从请求头提取
    request
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// 提取外部 trace_id
fn extract_trace_id(request: &Request) -> Option<String> {
    // 优先尝试 X-Trace-Id
    if let Some(trace_id) = request
        .headers()
        .get(X_TRACE_ID)
        .and_then(|v| v.to_str().ok())
    {
        return Some(trace_id.to_string());
    }

    // 尝试 W3C traceparent 格式: version-trace_id-parent_id-flags
    if let Some(traceparent) = request
        .headers()
        .get(TRACEPARENT)
        .and_then(|v| v.to_str().ok())
    {
        let parts: Vec<&str> = traceparent.split('-').collect();
        if parts.len() >= 2 {
            return Some(parts[1].to_string());
        }
    }

    None
}

/// 用户上下文扩展 trait
pub trait SpanExt {
    /// 注入用户 ID 到当前 span
    fn record_user_id(&self, user_id: &str);
    /// 注入用户名到当前 span
    fn record_username(&self, username: &str);
}

impl SpanExt for Span {
    fn record_user_id(&self, user_id: &str) {
        self.record("user_id", user_id);
    }

    fn record_username(&self, username: &str) {
        self.record("username", username);
    }
}

/// 创建带用户上下文的 span
#[macro_export]
macro_rules! user_span {
    ($level:ident, $name:expr, $user_id:expr) => {
        tracing::$level!($name, user_id = %$user_id)
    };
    ($level:ident, $name:expr, $user_id:expr, $($field:tt)*) => {
        tracing::$level!($name, user_id = %$user_id, $($field)*)
    };
}
