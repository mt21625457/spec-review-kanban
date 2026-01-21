# Change: 添加统一日志系统

## Why

当前项目的日志系统存在以下问题：

1. **缺乏统一配置**：aicodex 和 crates/server 各自独立初始化日志，配置分散
2. **无结构化日志**：仅使用基本的 fmt 输出，缺乏 JSON 格式支持
3. **无日志持久化**：所有日志仅输出到 stdout，无文件存储和轮转
4. **缺乏请求追踪**：HTTP 请求缺乏统一的 request_id 追踪能力
5. **日志上下文不完整**：缺乏用户、实例等业务上下文信息
6. **生产环境不友好**：无日志采样、无性能敏感日志控制

## What Changes

- 创建统一的日志模块 `crates/utils/src/logging/`
- 实现多 sink 输出：控制台（开发）、JSON 文件（生产）、Sentry（错误）
- 添加日志轮转机制（按时间：每日/每小时）
- 实现请求追踪 middleware，自动注入 request_id
- 提供业务上下文宏和 span 扩展
- 支持动态日志级别调整（运行时）
- 添加敏感信息脱敏功能

## Impact

- 受影响的规范：新增 `specs/logging-system/spec.md`
- 受影响的代码：
  - `crates/utils/src/` - 新增 logging 模块
  - `crates/server/src/main.rs` - 统一使用新日志系统
  - `aicodex/src/main.rs` - 统一使用新日志系统
  - `crates/server/src/routes/` - 添加请求追踪 middleware
- 依赖变更：
  - 新增 `tracing-appender`（日志文件输出和轮转）
