# Change: 迁移所有服务到统一日志系统

## Why

当前项目中有多个服务使用不同的日志初始化方式：
- `crates/server` - 已迁移到 `utils::logging`
- `aicodex` - 已迁移到 `utils::logging`
- `crates/server/src/bin/mcp_task_server.rs` - 直接使用 `tracing_subscriber`
- `crates/remote/src/lib.rs` - 自定义 `init_tracing()` 函数
- `crates/review/src/main.rs` - 直接使用 `tracing_subscriber::fmt()`

这导致：
1. 日志格式不一致
2. 配置分散，难以统一管理
3. 重复的 Sentry 集成代码
4. 无法统一启用文件输出、日志轮转等功能

## What Changes

- 扩展 `utils::logging` 支持新配置选项（ErrorLayer、Stderr、with_target、with_span_events）
- 迁移 `mcp_task_server` 到统一日志系统
- 迁移 `crates/remote` 日志初始化，移除 `init_tracing()` 和 `sentry_layer()`
  - **保留** `sentry_init_once()` 和 `configure_user_scope()`（使用独立 Sentry DSN）
- 迁移 `crates/review` 到统一日志系统
- 统一所有服务的日志配置方式

## Impact

- Affected specs: `logging-system`
- Affected code:
  - `crates/utils/Cargo.toml` (添加 tracing-error 依赖)
  - `crates/utils/src/logging/mod.rs` (扩展 API)
  - `crates/utils/src/logging/config.rs` (添加配置选项)
  - `crates/server/src/bin/mcp_task_server.rs`
  - `crates/remote/src/lib.rs` (移除 init_tracing, sentry_layer, sentry_init_once)
  - `crates/remote/src/main.rs` (更新日志初始化)
  - `crates/review/src/main.rs`
  - `crates/review/Cargo.toml` (添加 utils 依赖)
