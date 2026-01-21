pub mod config;
pub mod repo_mappings;
pub mod webhooks;
pub mod reviews;
pub mod proxy;

use axum::Router;

use crate::AppState;

/// 构建 API 路由
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/config", config::router())
        .nest("/repo-mappings", repo_mappings::router())
        .nest("/webhooks", webhooks::router())
        .nest("/reviews", reviews::router())
        .nest("/proxy", proxy::router())
}
