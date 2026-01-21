## ADDED Requirements

### Requirement: 统一日志初始化

所有服务 SHALL 使用 `utils::logging::init_logging()` 进行日志初始化。

#### Scenario: 标准服务初始化
- **WHEN** 服务启动时
- **THEN** 调用 `utils::logging::init_logging()` 初始化日志系统
- **AND** 使用 `LoggingConfig::builder()` 配置服务特定选项

#### Scenario: MCP 服务初始化
- **WHEN** MCP Task Server 启动时
- **THEN** 配置 `LogOutput::Stderr`（因为 stdout 用于 MCP 协议）
- **AND** 配置 `.level("debug")` 保持默认 debug 级别
- **AND** 启用 Sentry 集成

#### Scenario: Remote 服务初始化
- **WHEN** Remote 服务启动时
- **THEN** 配置 `LogFormat::Json` 格式输出
- **AND** 配置 `with_target(false)` 不显示 target
- **AND** 配置 `with_span_events(true)` 记录 span 关闭事件
- **AND** 配置 `enable_error_layer(true)` 增强错误追踪
- **AND** 启用 Sentry 集成

#### Scenario: CLI 工具初始化
- **WHEN** Review CLI 启动时
- **THEN** 根据 `--verbose` 参数配置日志级别（verbose=debug, 默认=warn）
- **AND** 使用 `LogFormat::Pretty` 输出到控制台

### Requirement: ErrorLayer 支持

日志系统 SHALL 支持可选的 `tracing_error::ErrorLayer` 集成。

#### Scenario: 启用 ErrorLayer
- **GIVEN** `LoggingConfig` 配置 `enable_error_layer = true`
- **WHEN** 日志系统初始化时
- **THEN** 添加 `ErrorLayer` 到 subscriber
- **AND** 错误追踪信息被记录

#### Scenario: 禁用 ErrorLayer
- **GIVEN** `LoggingConfig` 配置 `enable_error_layer = false`（默认）
- **WHEN** 日志系统初始化时
- **THEN** 不添加 `ErrorLayer`

### Requirement: Stderr 输出支持

日志系统 SHALL 支持输出到 stderr。

#### Scenario: 配置 stderr 输出
- **GIVEN** `LoggingConfig` 配置 `output = LogOutput::Stderr`
- **WHEN** 日志系统初始化时
- **THEN** 所有控制台日志输出到 stderr 而非 stdout

### Requirement: Target 显示配置

日志系统 SHALL 支持配置是否显示 target。

#### Scenario: 禁用 target 显示
- **GIVEN** `LoggingConfig` 配置 `with_target = false`
- **WHEN** 日志输出时
- **THEN** 日志行不包含 target 信息

#### Scenario: 启用 target 显示
- **GIVEN** `LoggingConfig` 配置 `with_target = true`（默认）
- **WHEN** 日志输出时
- **THEN** 日志行包含 target 信息

### Requirement: Span 事件配置

日志系统 SHALL 支持配置是否记录 span 事件。

#### Scenario: 启用 span 事件
- **GIVEN** `LoggingConfig` 配置 `with_span_events = true`
- **WHEN** span 关闭时
- **THEN** 记录 span 关闭事件

#### Scenario: 禁用 span 事件
- **GIVEN** `LoggingConfig` 配置 `with_span_events = false`（默认）
- **WHEN** span 关闭时
- **THEN** 不记录 span 关闭事件

## REMOVED Requirements

### Requirement: 分散的日志初始化

**Reason**: 统一使用 `utils::logging` 模块，移除各服务中重复的日志初始化代码。

**Migration**:
- 移除 `crates/remote/src/lib.rs` 中的 `init_tracing()` 函数
- 移除 `crates/remote/src/lib.rs` 中的 `sentry_layer()` 函数
- **保留** `crates/remote/src/lib.rs` 中的 `sentry_init_once()` 函数（使用独立 Sentry DSN）
- **保留** `crates/remote/src/lib.rs` 中的 `configure_user_scope()` 函数（参数类型不同）
- 更新所有服务使用 `utils::logging::init_logging()`
