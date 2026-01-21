# 日志系统技术设计

## Context

### 背景

本项目是一个 Rust Web 应用，包含多个服务模块（aicodex、crates/server、crates/remote 等）。当前使用 `tracing` + `tracing-subscriber` 的基础配置，仅支持控制台输出。随着系统复杂度增加，需要一个更完善的日志系统来支持：

- 生产环境问题排查
- 性能监控和分析
- 安全审计
- 用户行为分析

### 约束条件

1. **兼容性**：必须与现有 `tracing` 生态兼容，避免大规模重构
2. **性能**：日志系统不能成为性能瓶颈，异步写入是必须的
3. **可观测性**：需要与 Sentry 现有集成保持兼容
4. **跨平台**：支持 macOS、Linux、Windows

### 利益相关者

- 开发团队：需要详细的开发调试日志
- 运维团队：需要结构化日志便于 ELK/Loki 等系统采集
- 安全团队：需要审计日志和敏感信息脱敏

---

## Rust 日志框架对比分析

### 1. log（标准 facade）

**仓库**: https://github.com/rust-lang/log

**特点**:
- Rust 官方日志 facade，定义标准接口
- 简单轻量，编译时开销极小
- 需要配合具体实现（env_logger、fern 等）

**优点**:
- 生态广泛，几乎所有 crate 都支持
- API 简单：`log::info!("message")`
- 零成本抽象，未启用的日志级别在编译时消除

**缺点**:
- 不支持结构化日志（仅字符串消息）
- 不支持异步 span/context 追踪
- 无法携带结构化字段

**适用场景**: 简单 CLI 工具、库开发

---

### 2. env_logger

**仓库**: https://github.com/rust-cli/env_logger

**特点**:
- log facade 的常用实现
- 通过环境变量 `RUST_LOG` 配置

**优点**:
- 配置简单，开箱即用
- 启动快，依赖少

**缺点**:
- 仅支持控制台输出
- 不支持文件输出和轮转
- 不支持结构化日志

**适用场景**: 快速原型、简单 CLI

---

### 3. tracing（当前使用）

**仓库**: https://github.com/tokio-rs/tracing

**特点**:
- Tokio 生态的异步诊断框架
- 支持 span（作用域）和 event（事件）
- 结构化字段支持

**优点**:
- **异步友好**：专为 async/await 设计，span 可跨 await 点传播
- **结构化日志**：`tracing::info!(user_id = 123, "User logged in")`
- **丰富的生态**：tracing-subscriber、tracing-opentelemetry、tracing-appender 等
- **性能优秀**：禁用的 span/event 接近零开销
- **可组合性**：Layer 架构允许灵活组合多个输出

**缺点**:
- 学习曲线较 log 陡峭
- 配置较复杂

**适用场景**: 异步 Web 服务、微服务、需要分布式追踪的系统

---

### 4. slog

**仓库**: https://github.com/slog-rs/slog

**特点**:
- 早期的结构化日志框架
- 类似 Go 的 zap/zerolog 设计

**优点**:
- 结构化日志支持好
- 性能优秀
- 灵活的 drain 架构

**缺点**:
- 与 async 生态集成不如 tracing
- 社区活跃度下降
- 与 tracing 生态重叠，维护两套系统成本高

**适用场景**: 同步代码为主的项目

---

### 5. log4rs

**仓库**: https://github.com/estk/log4rs

**特点**:
- 类似 Java log4j 的日志框架
- 基于 YAML/JSON 配置文件

**优点**:
- 配置灵活（YAML/JSON/代码）
- 内置多种 appender（文件、滚动文件、控制台）
- 熟悉 log4j 的开发者容易上手

**缺点**:
- 基于 log facade，不支持 span/trace
- 与 tracing 生态不兼容
- 需要额外的配置文件

**适用场景**: 传统日志需求、熟悉 log4j 的团队

---

### 6. fern

**仓库**: https://github.com/daboross/fern

**特点**:
- log facade 的灵活实现
- 纯代码配置

**优点**:
- 配置灵活，支持多输出
- 支持颜色输出
- 轻量级

**缺点**:
- 基于 log facade，不支持结构化字段
- 文件轮转需要额外实现

**适用场景**: 需要灵活配置但不需要结构化日志的项目

---

### 框架对比总结

| 特性 | log + env_logger | tracing | slog | log4rs | fern |
|------|-----------------|---------|------|--------|------|
| 结构化日志 | ❌ | ✅ | ✅ | ❌ | ❌ |
| 异步支持 | ❌ | ✅✅ | ⚠️ | ❌ | ❌ |
| Span/Trace | ❌ | ✅ | ❌ | ❌ | ❌ |
| 文件输出 | ❌ | ✅ (appender) | ✅ | ✅ | ✅ |
| 日志轮转 | ❌ | ✅ (rolling) | ✅ | ✅ | ⚠️ |
| Sentry 集成 | ⚠️ | ✅ | ⚠️ | ❌ | ❌ |
| 性能 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| 学习曲线 | ⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ |
| 社区活跃度 | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐ |

---

## Goals / Non-Goals

### Goals

1. **统一日志初始化**：提供单一入口点，所有服务使用相同配置
2. **结构化日志**：支持 JSON 格式输出，便于日志采集系统解析
3. **多输出支持**：同时支持控制台（开发）和文件（生产）
4. **日志轮转**：支持按时间轮转（每日/每小时），配合文件清理防止磁盘爆满
5. **请求追踪**：HTTP 请求自动注入 request_id
6. **敏感信息脱敏**：密码、token 等自动脱敏
7. **动态配置**：支持运行时调整日志级别

### Non-Goals

1. **分布式追踪**：暂不实现 OpenTelemetry 集成（可后续扩展）
2. **日志聚合**：不实现日志收集/转发（由外部系统处理）
3. **日志分析**：不实现日志查询/分析功能

---

## Decisions

### Decision 1: 继续使用 tracing 生态

**选择**: 基于现有 `tracing` + `tracing-subscriber` 扩展，不迁移到其他框架

**理由**:
1. 项目已广泛使用 tracing，迁移成本高
2. tracing 是 async Rust 的事实标准，与 Tokio 生态深度集成
3. 结构化日志和 span 追踪是现代可观测性的基础
4. 丰富的 Layer 生态支持各种输出需求

**替代方案考虑**:
- slog：与 tracing 功能重叠，迁移收益不明显
- log4rs：不支持 span，功能弱于 tracing

---

### Decision 2: 使用 tracing-appender 实现文件输出

**选择**: `tracing-appender` 配合 `tracing-subscriber` 的 Layer

**理由**:
1. 官方维护，与 tracing 完美集成
2. 内置 rolling file 支持（按时间：DAILY/HOURLY）
3. 非阻塞写入，不影响主线程性能

**注意**: `tracing-appender` 原生不支持按文件大小轮转和自动删除旧文件。需要额外实现文件清理逻辑（定时任务或启动时清理）。

**配置示例**:
```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::new(
    Rotation::DAILY,
    "/var/log/aicodex",
    "app.log"
);

let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
```

---

### Decision 3: JSON 格式使用 tracing-subscriber 内置 json layer

**选择**: 使用 `tracing_subscriber::fmt::layer().json()` 而非 bunyan-formatter

**理由**:
1. 官方内置，无额外依赖
2. 输出格式符合 ELK/Loki 等系统要求
3. 性能更好（少一层转换）

**JSON 格式示例**:
```json
{
  "timestamp": "2025-01-21T10:30:00.000Z",
  "level": "INFO",
  "target": "aicodex::routes::auth",
  "message": "User login successful",
  "fields": {
    "user_id": "uuid-xxx",
    "request_id": "req-123"
  },
  "span": {
    "name": "handle_login"
  }
}
```

---

### Decision 4: 请求追踪使用 tower-http + tracing

**选择**: 使用 `tower_http::trace::TraceLayer` 配合自定义 `MakeSpan`

**理由**:
1. 已在使用 tower-http，无需额外依赖
2. 可自定义 span 字段（request_id、user_id 等）
3. 自动记录请求持续时间

**实现示例**:
```rust
use tower_http::trace::{TraceLayer, MakeSpan};
use tracing::Span;

#[derive(Clone)]
struct CustomMakeSpan;

impl<B> MakeSpan<B> for CustomMakeSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> Span {
        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            request_id = %request_id,
        )
    }
}
```

---

### Decision 5: 敏感信息脱敏使用自定义 FormatEvent

**选择**: 实现自定义 `tracing_subscriber::fmt::FormatEvent` 在格式化阶段进行脱敏

**理由**:
1. tracing Layer 的 Visitor 是只读的，无法直接修改字段值
2. 在 FormatEvent 阶段可以控制最终输出的字符串
3. 可配置脱敏规则
4. 对业务代码透明

**实现方案**:
```rust
use std::fmt;
use tracing_subscriber::fmt::{format, FormatEvent, FormatFields, MakeWriter};
use regex::Regex;

struct RedactingFormat<F> {
    inner: F,
    redact_patterns: Vec<Regex>,
}

impl<S, N, F> FormatEvent<S, N> for RedactingFormat<F>
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
    F: FormatEvent<S, N>,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        // 1. 先写入内部 buffer
        let mut buffer = String::new();
        let buf_writer = format::Writer::new(&mut buffer);
        self.inner.format_event(ctx, buf_writer, event)?;

        // 2. 应用脱敏规则
        let redacted = self.redact_patterns.iter().fold(buffer, |acc, pattern| {
            pattern.replace_all(&acc, "[REDACTED]").to_string()
        });

        // 3. 写入最终输出
        write!(writer, "{}", redacted)
    }
}
```

**脱敏策略**:
- 字段名包含 `password`、`token`、`secret`、`key`、`credential` 的自动脱敏
- 值匹配敏感模式（JWT、API Key 格式）的自动脱敏
- 脱敏后显示为 `[REDACTED]`

---

## Risks / Trade-offs

### Risk 1: 日志文件 I/O 影响性能

**风险**: 高并发场景下日志写入可能成为瓶颈

**缓解措施**:
1. 使用 `tracing-appender::non_blocking` 异步写入
2. 设置合理的 buffer 大小
3. 生产环境适当提高日志级别（INFO 以上）

### Risk 2: 日志文件过大

**风险**: 长期运行导致磁盘空间不足

**缓解措施**:
1. 配置日志轮转（DAILY/HOURLY）
2. 启动时清理超过 `LOG_MAX_FILES` 的旧文件
3. 配合系统 logrotate 或容器日志管理
4. 生产环境建议使用外部日志收集器（如 Fluentd）

### Risk 3: 敏感信息泄露

**风险**: 脱敏规则不完善导致敏感信息泄露

**缓解措施**:
1. 默认严格脱敏规则
2. 代码审查关注日志输出
3. 提供 `#[sensitive]` 属性宏标记敏感字段

---

## Migration Plan

### Phase 1: 创建日志模块（不影响现有代码）

1. 在 `crates/utils/src/` 创建 `logging/` 模块
2. 实现 `LoggingConfig` 和 `init_logging()` 函数
3. 添加单元测试

### Phase 2: 迁移 aicodex

1. 替换 `aicodex/src/main.rs` 中的日志初始化
2. 添加请求追踪 middleware
3. 验证功能正常

### Phase 3: 迁移 crates/server

1. 替换 `crates/server/src/main.rs` 中的日志初始化
2. 与 Sentry 集成保持兼容
3. 验证功能正常

### Phase 4: 迁移其他服务

1. 迁移 `crates/remote`
2. 迁移 MCP 相关服务

### Rollback Plan

每个 Phase 可独立回滚：
- 回滚代码更改
- 恢复原有日志初始化代码
- 不需要数据迁移

---

## Open Questions

1. **日志存储位置**: 使用 `~/.aicodex/logs/` 还是系统目录 `/var/log/aicodex/`？
   - 建议：开发环境使用项目目录，生产环境支持配置

2. **日志保留时间**: 保留多少天的日志？
   - 建议：默认 7 天，可通过环境变量配置

3. **是否需要 OpenTelemetry 集成**: 是否需要支持分布式追踪？
   - 建议：作为后续扩展，本次不实现

---

## 推荐架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Code                         │
│   tracing::info!(user_id = %id, "User logged in");              │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                     tracing_subscriber::Registry                 │
└─────────────────────────────────────────────────────────────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        ▼                        ▼                        ▼
┌───────────────┐      ┌───────────────┐      ┌───────────────────┐
│  Console Layer │      │   JSON File   │      │   Sentry Layer    │
│  (Dev: pretty) │      │    Layer      │      │  (Errors only)    │
│  (Prod: JSON)  │      │  (Rolling)    │      │                   │
└───────────────┘      └───────────────┘      └───────────────────┘
        │                        │                        │
        ▼                        ▼                        ▼
    stdout/stderr           日志文件               Sentry 服务
                    /var/log/aicodex/app.log
```
