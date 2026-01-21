# Logging System Specification

## ADDED Requirements

### Requirement: 统一日志初始化

系统 SHALL 提供统一的日志初始化入口 `init_logging()`，所有服务模块必须使用此函数初始化日志系统。

#### Scenario: 服务启动时初始化日志

- **WHEN** 服务启动
- **THEN** 调用 `init_logging()` 初始化日志系统
- **AND** 日志系统根据配置创建相应的输出 layers
- **AND** 返回必要的 guards 用于保持后台写入线程

#### Scenario: 配置来源优先级

- **WHEN** 初始化日志系统
- **THEN** 配置按以下优先级读取：代码显式配置 > 环境变量 > 默认值

---

### Requirement: 多输出目标支持

系统 SHALL 支持同时输出日志到多个目标：控制台、文件、Sentry。

#### Scenario: 开发环境日志输出

- **WHEN** 环境变量 `LOG_OUTPUT=console` 或未设置
- **THEN** 日志输出到 stdout/stderr
- **AND** 使用 pretty 格式（带颜色）
- **AND** 显示完整的 span 上下文

#### Scenario: 生产环境日志输出

- **WHEN** 环境变量 `LOG_OUTPUT=file` 或 `LOG_OUTPUT=both`
- **THEN** 日志输出到指定目录的文件
- **AND** 使用 JSON 格式
- **AND** 文件按配置的策略轮转

#### Scenario: 错误日志发送到 Sentry

- **WHEN** 日志级别为 ERROR 或 WARN
- **AND** Sentry 已初始化
- **THEN** ERROR 级别日志作为 Event 发送到 Sentry
- **AND** WARN 级别日志作为 Breadcrumb 记录

---

### Requirement: 结构化日志格式

系统 SHALL 支持结构化日志输出，包含必要的元数据字段。

#### Scenario: JSON 格式日志输出

- **WHEN** `LOG_FORMAT=json`
- **THEN** 每条日志输出为单行 JSON
- **AND** 包含以下字段：
  - `timestamp`: ISO 8601 格式时间戳
  - `level`: 日志级别（TRACE/DEBUG/INFO/WARN/ERROR）
  - `target`: 日志来源模块
  - `message`: 日志消息
  - `fields`: 结构化字段对象
  - `span`: 当前 span 信息（可选）

#### Scenario: Pretty 格式日志输出

- **WHEN** `LOG_FORMAT=pretty` 或未设置
- **THEN** 日志输出为人类可读格式
- **AND** 包含颜色高亮（ERROR 红色、WARN 黄色等）
- **AND** 显示时间、级别、模块、消息

---

### Requirement: 日志文件轮转

系统 SHALL 支持日志文件按时间自动轮转，并提供旧文件清理机制。

#### Scenario: 按时间轮转

- **WHEN** `LOG_ROTATION=daily`
- **THEN** 每天创建新的日志文件
- **AND** 文件名格式为 `app.YYYY-MM-DD.log`

#### Scenario: 按小时轮转

- **WHEN** `LOG_ROTATION=hourly`
- **THEN** 每小时创建新的日志文件
- **AND** 文件名格式为 `app.YYYY-MM-DD-HH.log`

#### Scenario: 旧文件清理

- **WHEN** 服务启动时
- **AND** 日志目录中文件数量超过 `LOG_MAX_FILES`
- **THEN** 删除最旧的日志文件直到数量符合限制
- **AND** 记录清理操作日志

---

### Requirement: HTTP 请求追踪

系统 SHALL 为每个 HTTP 请求生成唯一的 request_id，并在整个请求处理过程中传播。

#### Scenario: 请求 ID 生成

- **WHEN** 收到新的 HTTP 请求
- **THEN** 生成唯一的 UUID 作为 request_id
- **AND** 创建包含 request_id 的 tracing span
- **AND** 请求处理过程中所有日志自动包含此 request_id

#### Scenario: 请求 ID 响应

- **WHEN** HTTP 请求处理完成
- **THEN** 在响应 header 中包含 `X-Request-Id` 字段
- **AND** 值为该请求的 request_id

#### Scenario: 外部 trace_id 传播

- **WHEN** 请求 header 包含 `X-Trace-Id` 或 `traceparent`
- **THEN** 使用该值作为 trace_id
- **AND** 在日志中记录 trace_id 用于分布式追踪

#### Scenario: 请求日志记录

- **WHEN** HTTP 请求开始
- **THEN** 记录 INFO 级别日志，包含：
  - 请求方法（GET/POST 等）
  - 请求路径
  - 客户端 IP
  - User-Agent（可选）
- **WHEN** HTTP 请求完成
- **THEN** 记录日志，包含：
  - 响应状态码
  - 请求处理耗时（毫秒）

---

### Requirement: 敏感信息脱敏

系统 SHALL 自动对日志中的敏感信息进行脱敏处理。

#### Scenario: 敏感字段名脱敏

- **WHEN** 日志字段名包含以下关键词（不区分大小写）：
  - `password`
  - `token`
  - `secret`
  - `key`
  - `credential`
  - `authorization`
  - `api_key`
  - `apikey`
- **THEN** 字段值替换为 `[REDACTED]`

#### Scenario: 敏感值模式脱敏

- **WHEN** 日志字段值匹配以下模式：
  - JWT token 格式（以 `eyJ` 开头，包含两个 `.` 分隔符）
  - Bearer token 格式（`Bearer ` 前缀）
  - 常见 API Key 格式（32+ 字符的十六进制或 Base64）
- **THEN** 值替换为 `[REDACTED:<type>]`
- **AND** `<type>` 表示检测到的敏感类型（JWT/BEARER/API_KEY）

#### Scenario: 脱敏规则可配置

- **WHEN** 配置 `LOG_REDACT_PATTERNS` 环境变量
- **THEN** 使用配置的正则表达式进行额外脱敏
- **AND** 默认规则仍然生效

---

### Requirement: 日志级别配置

系统 SHALL 支持灵活的日志级别配置。

#### Scenario: 全局日志级别

- **WHEN** 设置 `LOG_LEVEL=info`
- **THEN** 只输出 INFO 及以上级别的日志
- **AND** DEBUG 和 TRACE 级别日志被过滤

#### Scenario: 模块级别配置

- **WHEN** 设置 `RUST_LOG=warn,aicodex=debug,tower_http=info`
- **THEN** 默认级别为 WARN
- **AND** aicodex 模块级别为 DEBUG
- **AND** tower_http 模块级别为 INFO

#### Scenario: 日志级别优先级

- **WHEN** 同时设置 `LOG_LEVEL` 和 `RUST_LOG`
- **THEN** `RUST_LOG` 的配置优先
- **AND** `LOG_LEVEL` 作为未指定模块的默认级别

---

### Requirement: 环境变量配置

系统 SHALL 支持通过环境变量配置日志系统。

#### Scenario: 支持的环境变量

系统 SHALL 支持以下环境变量：

| 变量名 | 默认值 | 描述 |
|--------|--------|------|
| `LOG_LEVEL` | `info` | 默认日志级别 |
| `LOG_FORMAT` | `pretty` | 输出格式：pretty/json |
| `LOG_OUTPUT` | `console` | 输出目标：console/file/both |
| `LOG_DIR` | `./logs` | 日志文件目录 |
| `LOG_MAX_FILES` | `7` | 最大保留文件数 |
| `LOG_ROTATION` | `daily` | 轮转策略：daily/hourly |
| `LOG_REDACT_PATTERNS` | - | 自定义脱敏正则表达式（逗号分隔） |
| `RUST_LOG` | - | 细粒度日志级别配置 |

#### Scenario: 无效配置处理

- **WHEN** 环境变量值无效
- **THEN** 使用默认值
- **AND** 在初始化时输出 WARN 日志提示配置无效

---

### Requirement: 错误处理

系统 SHALL 优雅地处理日志系统相关的错误，不影响主服务运行。

#### Scenario: 日志目录不存在

- **WHEN** 配置的日志目录不存在
- **THEN** 尝试创建目录
- **AND** 如果创建失败，回退到控制台输出
- **AND** 输出 WARN 日志说明原因

#### Scenario: 无写入权限

- **WHEN** 日志目录无写入权限
- **THEN** 回退到控制台输出
- **AND** 输出 ERROR 日志说明权限问题

#### Scenario: 磁盘空间不足

- **WHEN** 写入日志时磁盘空间不足
- **THEN** 继续尝试写入（由操作系统处理）
- **AND** 不影响主服务运行
- **AND** 依赖外部监控发现磁盘问题
