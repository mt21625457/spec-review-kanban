## 0. 扩展 utils::logging API（前置任务）

- [ ] 0.1 添加 `tracing-error` 依赖到 `crates/utils/Cargo.toml`
- [ ] 0.2 在 `LoggingConfig` 中添加 `enable_error_layer: bool` 选项（默认 false）
- [ ] 0.3 在 `LoggingConfig` 中添加 `with_target: bool` 选项（默认 true）
- [ ] 0.4 在 `LoggingConfig` 中添加 `with_span_events: bool` 选项（默认 false）
- [ ] 0.5 在 `LoggingConfigBuilder` 中添加对应的 builder 方法
- [ ] 0.6 更新 `init_logging()` 支持可选的 `ErrorLayer`
- [ ] 0.7 更新 fmt layer 支持 `with_target` 和 `with_span_events` 配置
- [ ] 0.8 添加 `LogOutput::Stderr` 变体支持 stderr 输出
- [ ] 0.9 编译验证 utils crate

## 1. 迁移 MCP Task Server

- [ ] 1.1 更新 `crates/server/src/bin/mcp_task_server.rs` 使用 `utils::logging`
- [ ] 1.2 移除直接的 `tracing_subscriber` 导入
- [ ] 1.3 配置 `LogOutput::Stderr`（stdout 用于 MCP 协议通信）
- [ ] 1.4 配置 `.level("debug")` 保持默认 debug 级别
- [ ] 1.5 配置 `enable_sentry(true)` 保持 Sentry 集成
- [ ] 1.6 编译验证 mcp_task_server

## 2. 迁移 Remote 服务

**注意**: Remote 服务使用独立的 Sentry DSN，保留 `sentry_init_once` 和 `configure_user_scope`。

- [ ] 2.1 更新 `crates/remote/src/main.rs` 使用 `utils::logging::init_logging()`
- [ ] 2.2 配置 `LogFormat::Json` 保持 JSON 格式输出
- [ ] 2.3 配置 `with_target(false)` 保持不显示 target
- [ ] 2.4 配置 `with_span_events(true)` 保持记录 span 关闭事件
- [ ] 2.5 配置 `enable_error_layer(true)` 保持 ErrorLayer 功能
- [ ] 2.6 配置 `enable_sentry(true)` 启用 Sentry layer（使用 remote 自己的 sentry_layer）
- [ ] 2.7 移除 `crates/remote/src/lib.rs` 中的 `init_tracing()` 函数
- [ ] 2.8 **保留** `crates/remote/src/lib.rs` 中的 `sentry_init_once()` 函数（独立 DSN）
- [ ] 2.9 **保留** `crates/remote/src/lib.rs` 中的 `configure_user_scope()` 函数（参数类型不同）
- [ ] 2.10 移除 `crates/remote/src/lib.rs` 中的 `sentry_layer()` 函数（使用 utils 版本）
- [ ] 2.11 编译验证 remote crate

## 3. 迁移 Review CLI

- [ ] 3.1 添加 utils crate 依赖到 `crates/review/Cargo.toml`
- [ ] 3.2 更新 `crates/review/src/main.rs` 使用 `utils::logging`
- [ ] 3.3 根据 `--verbose` 参数动态设置日志级别（verbose=debug, 默认=warn）
- [ ] 3.4 配置 `LogOutput::Console` 和 `LogFormat::Pretty`
- [ ] 3.5 移除直接的 `tracing_subscriber` 导入
- [ ] 3.6 编译验证 review crate

## 4. 验证

- [ ] 4.1 运行 `cargo build --workspace` 确保全部编译通过
- [ ] 4.2 运行 `cargo test -p utils logging` 确保日志模块测试通过
- [ ] 4.3 手动验证 MCP Server 日志输出到 stderr
- [ ] 4.4 手动验证 Remote 服务：JSON 格式、无 target、有 span 事件
- [ ] 4.5 手动验证 Review CLI verbose 模式
- [ ] 4.6 验证 Remote 服务 Sentry 仍使用独立 DSN
