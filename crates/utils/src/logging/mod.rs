//! 统一日志系统模块
//!
//! 提供统一的日志初始化、配置和中间件支持。
//!
//! # 使用方式
//!
//! ```rust,ignore
//! use utils::logging::{LoggingConfig, init_logging};
//!
//! // 使用默认配置（从环境变量读取）
//! let _guard = init_logging(LoggingConfig::from_env())?;
//!
//! // 使用构建器自定义配置
//! let config = LoggingConfig::builder()
//!     .level("debug")
//!     .format(LogFormat::Json)
//!     .output(LogOutput::Both)
//!     .service_name("my-service")
//!     .with_env()  // 环境变量覆盖
//!     .build();
//! let _guard = init_logging(config)?;
//! ```
//!
//! # 环境变量
//!
//! - `LOG_LEVEL`: 默认日志级别 (default: "info")
//! - `LOG_FORMAT`: 输出格式 pretty/json (default: "pretty")
//! - `LOG_OUTPUT`: 输出目标 console/file/both (default: "console")
//! - `LOG_DIR`: 日志文件目录 (default: "./logs")
//! - `LOG_MAX_FILES`: 最大保留文件数 (default: 7)
//! - `LOG_ROTATION`: 轮转策略 daily/hourly (default: "daily")
//! - `LOG_REDACT_PATTERNS`: 自定义脱敏正则表达式（逗号分隔）
//! - `RUST_LOG`: 细粒度日志级别配置

pub mod cleanup;
pub mod config;
pub mod layers;
pub mod middleware;
pub mod redact;

pub use config::{LogFormat, LogOutput, LogRotation, LoggingConfig, LoggingConfigBuilder};
pub use middleware::{request_tracing_middleware, SpanExt, X_REQUEST_ID, X_TRACE_ID};
pub use redact::Redactor;

use std::io;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::sentry::sentry_layer;
use cleanup::{check_write_permission, cleanup_old_logs, ensure_log_dir};

/// 日志系统初始化错误
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("日志目录创建失败: {0}")]
    DirectoryCreation(#[from] io::Error),
    #[error("日志目录无写入权限: {0}")]
    PermissionDenied(String),
    #[error("日志系统初始化失败: {0}")]
    InitError(String),
}

/// 日志系统 Guard
///
/// 持有非阻塞写入线程的 guard，确保日志完整写入。
/// 当 guard 被 drop 时，会等待所有日志写入完成。
pub struct LoggingGuard {
    _file_guard: Option<WorkerGuard>,
}

/// 初始化日志系统
///
/// 根据配置创建相应的 layers 并注册到 tracing_subscriber。
///
/// # Arguments
///
/// * `config` - 日志配置
///
/// # Returns
///
/// 返回 `LoggingGuard`，必须保持其生命周期直到程序结束。
///
/// # Errors
///
/// 如果日志目录创建失败或无写入权限，返回错误。
/// 在错误情况下，会尝试回退到控制台输出。
pub fn init_logging(config: LoggingConfig) -> Result<LoggingGuard, LoggingError> {
    let env_filter = build_env_filter(&config);

    // 仅控制台输出
    if !config.should_output_file() {
        return init_console_only(config, env_filter);
    }

    // 检查并创建日志目录
    if let Err(e) = ensure_log_dir(&config.log_dir) {
        eprintln!("警告: 无法创建日志目录 {:?}: {}，回退到控制台输出", config.log_dir, e);
        return init_console_only(config, env_filter);
    }

    if !check_write_permission(&config.log_dir).unwrap_or(false) {
        eprintln!("错误: 日志目录 {:?} 无写入权限，回退到控制台输出", config.log_dir);
        return init_console_only(config, env_filter);
    }

    // 清理旧日志文件
    if let Err(e) = cleanup_old_logs(&config.log_dir, &config.service_name, config.max_files) {
        eprintln!("警告: 清理旧日志文件失败: {}", e);
    }

    // 创建文件 appender
    let rotation = match config.rotation {
        LogRotation::Daily => Rotation::DAILY,
        LogRotation::Hourly => Rotation::HOURLY,
    };

    let file_appender = RollingFileAppender::new(
        rotation,
        &config.log_dir,
        format!("{}.log", config.service_name),
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 根据输出模式初始化
    if config.should_output_console() {
        // 同时输出到控制台和文件
        init_both(config, env_filter, non_blocking, guard)
    } else {
        // 仅文件输出
        init_file_only(config, env_filter, non_blocking, guard)
    }
}

/// 获取 span 事件配置
fn get_span_events(config: &LoggingConfig) -> FmtSpan {
    if config.with_span_events {
        FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    }
}

/// 仅控制台输出初始化
fn init_console_only(config: LoggingConfig, env_filter: EnvFilter) -> Result<LoggingGuard, LoggingError> {
    let span_events = get_span_events(&config);
    let with_target = config.with_target;
    let use_stderr = config.should_output_stderr();
    let use_json = config.is_json_format();
    let use_error_layer = config.enable_error_layer;
    let use_sentry = config.enable_sentry;

    // 根据配置组合初始化
    // 由于 Rust 类型系统限制，需要为每种组合单独处理
    match (use_json, use_stderr, use_error_layer, use_sentry) {
        // JSON + stderr
        (true, true, true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, true, true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, true, false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, true, false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        // JSON + stdout
        (true, false, true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, false, true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, false, false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, false, false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        // Pretty + stderr
        (false, true, true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, true, true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, true, false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, true, false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_writer(std::io::stderr).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        // Pretty + stdout
        (false, false, true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, false, true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, false, false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, false, false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(span_events).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
    }

    log_init_complete(&config);
    Ok(LoggingGuard { _file_guard: None })
}

/// 仅文件输出初始化
fn init_file_only(
    config: LoggingConfig,
    env_filter: EnvFilter,
    non_blocking: tracing_appender::non_blocking::NonBlocking,
    guard: WorkerGuard,
) -> Result<LoggingGuard, LoggingError> {
    let span_events = get_span_events(&config);
    let with_target = config.with_target;

    // 文件输出默认使用 JSON 格式
    match (config.enable_error_layer, config.enable_sentry) {
        (true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(non_blocking).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(non_blocking).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(non_blocking).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(span_events).with_writer(non_blocking).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
    }

    log_init_complete(&config);
    Ok(LoggingGuard { _file_guard: Some(guard) })
}

/// 同时输出到控制台和文件
fn init_both(
    config: LoggingConfig,
    env_filter: EnvFilter,
    non_blocking: tracing_appender::non_blocking::NonBlocking,
    guard: WorkerGuard,
) -> Result<LoggingGuard, LoggingError> {
    // 需要为每个 layer 创建独立的 filter
    let console_filter = build_env_filter(&config);
    let with_target = config.with_target;
    let use_stderr = config.should_output_stderr();

    // 控制台使用 pretty，文件使用 JSON
    // 由于类型系统限制，需要为每种组合单独处理
    // FmtSpan 不实现 Copy，所以需要为每个 layer 单独获取
    match (use_stderr, config.enable_error_layer, config.enable_sentry) {
        // stderr 输出
        (true, true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(std::io::stderr).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(std::io::stderr).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(std::io::stderr).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (true, false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(std::io::stderr).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        // stdout 输出
        (false, true, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .with(ErrorLayer::default())
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, true, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .with(ErrorLayer::default())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, false, true) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .with(sentry_layer())
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
        (false, false, false) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_ansi(true).with_target(with_target).with_span_events(get_span_events(&config)).with_filter(console_filter))
                .with(tracing_subscriber::fmt::layer().json().with_ansi(false).with_target(with_target).with_span_events(get_span_events(&config)).with_writer(non_blocking).with_filter(env_filter))
                .try_init().map_err(|e| LoggingError::InitError(e.to_string()))?;
        }
    }

    log_init_complete(&config);
    Ok(LoggingGuard { _file_guard: Some(guard) })
}

/// 构建 EnvFilter
fn build_env_filter(config: &LoggingConfig) -> EnvFilter {
    // 优先使用 RUST_LOG 环境变量
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        EnvFilter::try_new(&rust_log).unwrap_or_else(|_| {
            eprintln!("警告: 无效的 RUST_LOG 配置，使用默认级别 {}", config.level);
            EnvFilter::new(&config.level)
        })
    } else {
        EnvFilter::new(&config.level)
    }
}

/// 记录初始化完成日志
fn log_init_complete(config: &LoggingConfig) {
    tracing::info!(
        service = %config.service_name,
        level = %config.level,
        format = ?config.format,
        output = ?config.output,
        "日志系统初始化完成"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_from_env() {
        // 测试默认配置
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.should_output_console());
        assert!(!config.should_output_file());
    }

    #[test]
    fn test_logging_config_builder() {
        let config = LoggingConfig::builder()
            .level("debug")
            .format(LogFormat::Json)
            .output(LogOutput::Both)
            .service_name("test")
            .build();

        assert_eq!(config.level, "debug");
        assert!(config.is_json_format());
        assert!(config.should_output_console());
        assert!(config.should_output_file());
    }
}
