pub mod config;
pub mod repo_mappings;
pub mod webhooks;
pub mod reviews;
pub mod proxy;

// 多实例管理路由
pub mod auth;
pub mod users;
pub mod instances;
pub mod my_instances;

use axum::Router;

use crate::AppState;

/// 构建 API 路由
pub fn api_routes() -> Router<AppState> {
    // 公开路由（无需认证）
    let public_routes = Router::new()
        .nest("/auth", auth::auth_routes());

    // 需要认证的路由（中间件在 handler 层处理）
    let authenticated_routes = Router::new()
        .nest("/my-instances", my_instances::my_instances_routes());

    // 管理员路由（中间件在 handler 层处理）
    let admin_routes = Router::new()
        .nest("/users", users::users_routes())
        .nest("/instances", instances::instances_routes());

    // 原有路由（保持向后兼容）
    let legacy_routes = Router::new()
        .nest("/config", config::router())
        .nest("/repo-mappings", repo_mappings::router())
        .nest("/webhooks", webhooks::router())
        .nest("/reviews", reviews::router())
        .nest("/proxy", proxy::router());

    Router::new()
        .merge(public_routes)
        .merge(authenticated_routes)
        .merge(admin_routes)
        .merge(legacy_routes)
}
