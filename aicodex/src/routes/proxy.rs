use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};

use crate::AppState;

/// 代理请求到 Vibe Kanban
async fn proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // 读取请求体
    let body = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(bytes) => {
            if bytes.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&bytes).to_string())
            }
        }
        Err(_) => None,
    };

    // 调用 Vibe Kanban
    match state
        .services
        .vibe
        .proxy_request(method.as_str(), &path, body.as_deref())
        .await
    {
        Ok((status, response_body)) => {
            let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Response::builder()
                .status(status_code)
                .header("Content-Type", "application/json")
                .body(Body::from(response_body))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("代理响应构建失败"))
                        .unwrap()
                })
        }
        Err(e) => Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "success": false,
                    "error": format!("代理请求失败: {}", e),
                })
                .to_string(),
            ))
            .unwrap(),
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/*path", any(proxy_handler))
}
