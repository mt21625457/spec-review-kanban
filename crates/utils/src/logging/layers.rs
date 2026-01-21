//! 自定义 tracing layers 模块

use tracing_subscriber::fmt::format;
use tracing_subscriber::registry::LookupSpan;

/// 创建控制台 pretty 格式 layer
pub fn console_pretty_layer<S>() -> tracing_subscriber::fmt::Layer<S>
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
}

/// 创建控制台 JSON 格式 layer
pub fn console_json_layer<S>() -> tracing_subscriber::fmt::Layer<S, format::JsonFields, format::Format<format::Json>>
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_current_span(true)
}

/// 创建文件 JSON 格式 layer
pub fn file_json_layer<S, W>(
    writer: W,
) -> tracing_subscriber::fmt::Layer<S, format::JsonFields, format::Format<format::Json>, W>
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
    W: for<'writer> tracing_subscriber::fmt::MakeWriter<'writer> + 'static,
{
    tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_target(true)
        .with_current_span(true)
        .with_writer(writer)
}

/// 创建文件 pretty 格式 layer
pub fn file_pretty_layer<S, W>(
    writer: W,
) -> tracing_subscriber::fmt::Layer<S, format::DefaultFields, format::Format<format::Full>, W>
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
    W: for<'writer> tracing_subscriber::fmt::MakeWriter<'writer> + 'static,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_writer(writer)
}
