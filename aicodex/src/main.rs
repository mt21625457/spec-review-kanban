mod config;
mod db;
mod error;
mod middleware;
mod routes;
mod services;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use utils::logging::{LoggingConfig, LogOutput};

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
    // 加载环境变量（在日志初始化前，以便日志配置可以读取环境变量）
    dotenvy::dotenv().ok();

    // 初始化日志系统
    let logging_config = LoggingConfig::builder()
        .level("debug")
        .output(LogOutput::Console)
        .service_name("aicodex")
        .with_env()
        .build();
    let _logging_guard = utils::logging::init_logging(logging_config)
        .expect("日志系统初始化失败");

    // 加载配置
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

    // 配置静态文件服务（SPA 支持）
    let static_dir = std::env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../aicodex-web/dist"));

    let index_path = static_dir.join("index.html");

    // 构建路由
    let app = if static_dir.exists() && index_path.exists() {
        tracing::info!("启用静态文件服务: {:?}", static_dir);
        // 使用 SPA fallback：所有非 API 请求都回退到 index.html
        let serve_dir = ServeDir::new(&static_dir)
            .not_found_service(ServeFile::new(&index_path));

        Router::new()
            .nest("/api", routes::api_routes())
            .fallback_service(serve_dir)
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    } else {
        tracing::warn!("静态文件目录不存在: {:?}，仅提供 API 服务", static_dir);
        tracing::warn!("请先构建前端: cd aicodex-web && pnpm run build");
        Router::new()
            .nest("/api", routes::api_routes())
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    };

    // 启动服务器
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("服务监听于 http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
