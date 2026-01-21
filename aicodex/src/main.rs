mod config;
mod db;
mod error;
mod routes;
mod services;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::db::Database;
use crate::services::Services;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub services: Arc<Services>,
    pub config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aicodex=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;
    let config = Arc::new(config);

    tracing::info!("启动 aicodex 服务...");

    // 初始化数据库
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // 运行数据库迁移
    sqlx::migrate!("./migrations").run(&pool).await?;

    let db = Arc::new(Database::new(pool));

    // 初始化服务
    let services = Arc::new(Services::new(
        db.clone(),
        config.clone(),
    ));

    // 创建应用状态
    let state = AppState {
        db,
        services,
        config: config.clone(),
    };

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 启动服务器
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("服务监听于 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
