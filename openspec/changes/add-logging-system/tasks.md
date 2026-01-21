# 日志系统实施任务清单

## 1. 核心日志模块

- [ ] 1.1 在 `crates/utils/Cargo.toml` 添加依赖
  - `tracing-appender` - 文件输出和轮转
  - `regex` - 敏感信息脱敏模式匹配
  - 更新 `tracing-subscriber` features（json）

- [ ] 1.2 创建 `crates/utils/src/logging/mod.rs` 模块结构
  - `config.rs` - 配置结构体
  - `init.rs` - 初始化函数
  - `layers.rs` - 自定义 layers
  - `middleware.rs` - HTTP 请求追踪中间件
  - `redact.rs` - 敏感信息脱敏

- [ ] 1.3 实现 `LoggingConfig` 配置结构
  - 日志级别配置
  - 输出目标配置（console/file/both）
  - 文件路径和轮转配置
  - JSON 格式开关
  - 脱敏规则配置

- [ ] 1.4 实现 `init_logging()` 初始化函数
  - 根据配置创建 layers
  - 注册到 tracing_subscriber
  - 返回必要的 guards（用于保持后台写入线程）

- [ ] 1.5 实现文件输出 layer
  - 集成 tracing-appender
  - 支持日志轮转（按天/按小时）
  - **注意**: tracing-appender 不支持按大小轮转

- [ ] 1.6 实现旧日志文件清理
  - 服务启动时检查日志目录
  - 删除超过 `LOG_MAX_FILES` 的旧文件
  - 按文件修改时间排序

- [ ] 1.7 实现 JSON 格式 layer
  - 使用 tracing_subscriber::fmt::json()
  - 自定义字段格式
  - 时间戳格式化（ISO 8601）

## 2. 请求追踪

- [ ] 2.1 实现自定义 `MakeSpan` trait
  - 自动生成 request_id
  - 提取 HTTP 方法、路径、查询参数
  - 支持从 header 提取 trace_id（兼容分布式追踪）

- [ ] 2.2 创建请求追踪 middleware
  - 记录请求开始时间
  - 记录响应状态码
  - 计算请求持续时间
  - 在响应 header 中返回 request_id

- [ ] 2.3 实现用户上下文注入
  - 从认证信息提取 user_id
  - 注入到当前 span

## 3. 敏感信息脱敏

- [ ] 3.1 实现自定义 FormatEvent 包装器
  - 包装内部 FormatEvent
  - 在输出前应用脱敏规则

- [ ] 3.2 实现敏感模式匹配
  - JWT token 格式检测
  - API Key 格式检测
  - 密码字段检测

- [ ] 3.3 添加 `#[sensitive]` 属性宏（可选）
  - 编译时标记敏感字段
  - 自动脱敏支持

## 4. 集成迁移

- [ ] 4.1 迁移 aicodex
  - 替换 `aicodex/src/main.rs` 日志初始化
  - 添加请求追踪 middleware
  - 测试验证

- [ ] 4.2 迁移 crates/server
  - 替换 `crates/server/src/main.rs` 日志初始化
  - 保持 Sentry 集成
  - 测试验证

- [ ] 4.3 迁移 crates/remote
  - 替换日志初始化
  - 测试验证

## 5. 文档和测试

- [ ] 5.1 编写单元测试
  - 配置解析测试
  - 日志输出格式测试
  - 脱敏功能测试

- [ ] 5.2 编写集成测试
  - 文件轮转测试
  - 多服务并发日志测试

- [ ] 5.3 更新文档
  - 更新 CLAUDE.md 添加日志配置说明
  - 添加环境变量说明
  - 添加使用示例

## 6. 配置和运维

- [ ] 6.1 定义环境变量
  - `LOG_LEVEL` - 日志级别
  - `LOG_FORMAT` - 输出格式（pretty/json）
  - `LOG_OUTPUT` - 输出目标（console/file/both）
  - `LOG_DIR` - 日志文件目录
  - `LOG_MAX_FILES` - 最大保留文件数
  - `LOG_ROTATION` - 轮转策略（daily/hourly）

- [ ] 6.2 添加默认配置
  - 开发环境：console + pretty 格式
  - 生产环境：file + JSON 格式

- [ ] 6.3 实现运行时日志级别调整（可选）
  - 提供 API 端点动态调整
  - 或通过信号量触发
