use remote::{Server, config::RemoteServerConfig, sentry_init_once};
use utils::logging::{LogFormat, LoggingConfig, init_logging};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install rustls crypto provider before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // 初始化 Remote 专用的 Sentry（使用独立 DSN）
    sentry_init_once();

    // 使用统一日志系统
    // 注意：不启用 enable_sentry，因为 Remote 使用自己的 sentry_init_once
    let _guard = init_logging(
        LoggingConfig::builder()
            .level("info,sqlx=warn")
            .format(LogFormat::Json)
            .service_name("remote")
            .with_target(false)
            .with_span_events(true)
            .enable_error_layer(true)
            .with_env()
            .build(),
    )?;

    let config = RemoteServerConfig::from_env()?;
    Server::run(config).await
}
