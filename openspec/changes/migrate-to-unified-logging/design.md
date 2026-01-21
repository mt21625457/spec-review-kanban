## Context

项目已经实现了统一日志系统 (`utils::logging` 模块)，但部分服务仍使用旧的日志初始化方式。本次变更将所有服务迁移到统一日志系统。

### 当前状态

| 服务 | 当前方式 | 目标方式 |
|------|----------|----------|
| `crates/server` | `utils::logging` | ✅ 已迁移 |
| `aicodex` | `utils::logging` | ✅ 已迁移 |
| `mcp_task_server` | 直接 `tracing_subscriber` | 需迁移 |
| `crates/remote` | 自定义 `init_tracing()` | 需迁移 |
| `crates/review` | 直接 `tracing_subscriber::fmt()` | 需迁移 |

### 关键差异分析

**Remote 服务特殊性**:
1. 使用独立的 Sentry DSN（不同的 Sentry 项目）
2. `configure_user_scope` 参数类型为 `uuid::Uuid`（非 `&str`）
3. 使用 `.with_target(false)` 和 `.with_span_events(FmtSpan::CLOSE)`
4. 默认日志 filter: `"info,sqlx=warn"`

**MCP Server 特殊性**:
1. 日志输出到 stderr（stdout 用于协议通信）
2. 默认使用 debug 级别

## Goals / Non-Goals

### Goals

- 统一所有服务的日志初始化方式
- 保持各服务的特殊需求（如 MCP 的 stderr 输出、Remote 的 JSON 格式）
- 减少重复代码（如重复的 `sentry_layer()` 定义）
- 支持通过环境变量统一配置所有服务的日志

### Non-Goals

- 修改日志内容或业务逻辑
- 更改现有的日志级别默认值
- **合并 Remote 服务的 Sentry 项目**（保持独立）

## Decisions

### Decision 1: MCP Server stderr 输出

MCP Task Server 需要将日志输出到 stderr（因为 stdout 用于 MCP 协议通信）。

**方案 A**: 在 `LogOutput` 枚举中添加 `Stderr` 变体
**方案 B**: 在 MCP Server 中保持自定义初始化

**选择**: 方案 A - 添加 `LogOutput::Stderr` 变体。这是通用需求，其他 CLI 工具也可能需要。

### Decision 2: Remote 服务 ErrorLayer

Remote 服务使用 `tracing_error::ErrorLayer` 来增强错误追踪。

**方案**: 在统一日志系统中支持可选的 ErrorLayer。

**选择**: 在 `LoggingConfig` 中添加 `enable_error_layer` 选项。

### Decision 3: Remote 服务 Sentry 集成

Remote 服务使用独立的 Sentry DSN，不能直接使用 `utils::sentry::init_once`。

**方案 A**: 在 `utils::sentry` 中支持多个 DSN（通过 SentrySource 扩展）
**方案 B**: Remote 服务保留自己的 `sentry_init_once`，只迁移日志初始化

**选择**: 方案 B - Remote 服务保留 `sentry_init_once` 和 `configure_user_scope`。
原因：不同的 Sentry 项目是有意设计的，不应改变。只统一日志初始化部分。

### Decision 4: Review CLI 简化

Review CLI 是命令行工具，只需要简单的控制台输出。

**方案**: 直接使用 `LoggingConfig::builder()` 配置。

**选择**: 使用标准配置，通过 verbose flag 控制日志级别。

### Decision 5: 扩展日志配置选项

Remote 服务需要额外的配置选项。

**需要添加的选项**:
- `with_target: bool` - 是否显示 target（默认 true）
- `with_span_events: bool` - 是否记录 span 事件（默认 false）

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| 迁移后日志输出格式变化 | 仔细配置 `LogFormat` 和新增选项保持一致 |
| 特殊场景支持不足 | 为特殊场景提供自定义配置选项 |
| 依赖循环 | 确保 utils crate 不引入新的依赖循环 |
| Remote Sentry 配置丢失 | 保留 Remote 的 sentry_init_once |

## Migration Plan

1. **Phase 1**: 为特殊场景扩展 `utils::logging`（ErrorLayer、Stderr、with_target、with_span_events）
2. **Phase 2**: 逐个迁移服务，每次迁移后验证
3. **Phase 3**: 移除旧的重复代码（但保留 Remote 的 Sentry 相关代码）

回滚策略：每个服务的迁移独立进行，可单独回滚。
