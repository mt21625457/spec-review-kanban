# Design: Multi-Instance Vibe-Kanban Management

## Context

### 背景
vibe-kanban 是一个本地桌面应用，设计为单用户使用，数据存储在 `~/Library/Application Support/ai.bloop.vibe-kanban/db.sqlite`。aicodex 作为代理层，将前端请求转发给 vibe-kanban。

### 当前限制
1. 单实例架构无法支持多用户场景
2. 数据目录硬编码，无法动态配置
3. AI 智能体（Claude Code 等）共享同一套凭证和配置
4. 无法在服务器上运行多个隔离的 vibe-kanban 实例

### 利益相关者
- 运维团队：需要在服务器上管理多个实例
- 开发团队：需要隔离的开发环境
- 安全团队：需要 AI 智能体凭证隔离

## Architecture Overview

### 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              aicodex (控制中心)                              │
│                                                                             │
│  ┌───────────────────────┐      ┌─────────────────────────────────────┐    │
│  │    Instance Manager   │      │         Proxy Router                │    │
│  │                       │      │   /api/proxy/*                      │    │
│  │  - 创建/删除实例      │      │                                     │    │
│  │  - 启动/停止实例      │      │   基于用户会话自动路由到分配的实例  │    │
│  │  - 健康检查           │      └──────────────┬──────────────────────┘    │
│  │  - 自动恢复           │                     │                           │
│  │  - 端口分配           │                     │                           │
│  └───────────┬───────────┘                     │                           │
│              │                                 │                           │
│  ┌───────────┴───────────┐      ┌──────────────┴──────────────────────┐    │
│  │   aicodex Database    │      │        Agent Config Manager         │    │
│  │   (SQLite)            │      │                                     │    │
│  │                       │      │  - API Key 加密存储                 │    │
│  │  - users              │      │  - 环境变量注入                     │    │
│  │  - user_sessions      │      │  - 配置文件生成                     │    │
│  │  - vibe_instances     │      └─────────────────────────────────────┘    │
│  │  - instance_ai_agents │                                                 │
│  │  - instance_usage     │                                                 │
│  └───────────────────────┘                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                │ 控制 (启动/停止/监控)
                │
                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           子进程层 (vibe-kanban 实例)                        │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │   vibe-kanban       │  │   vibe-kanban       │  │   vibe-kanban       │ │
│  │   实例 A            │  │   实例 B            │  │   实例 C            │ │
│  │                     │  │                     │  │                     │ │
│  │   Port: 18100       │  │   Port: 18101       │  │   Port: 18102       │ │
│  │   Owner: 团队 A     │  │   Owner: 团队 B     │  │   Owner: 团队 C     │ │
│  │                     │  │                     │  │                     │ │
│  │   ┌───────────────┐ │  │   ┌───────────────┐ │  │   ┌───────────────┐ │ │
│  │   │ SQLite DB     │ │  │   │ SQLite DB     │ │  │   │ SQLite DB     │ │ │
│  │   │ Worktrees     │ │  │   │ Worktrees     │ │  │   │ Worktrees     │ │ │
│  │   │ AI Agents     │ │  │   │ AI Agents     │ │  │   │ AI Agents     │ │ │
│  │   └───────────────┘ │  │   └───────────────┘ │  │   └───────────────┘ │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
│                                                                             │
│  数据目录:                                                                   │
│  /data/vibe-instances/inst-a/    /data/vibe-instances/inst-b/    ...       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 核心职责划分

| 组件 | 职责 |
|------|------|
| **aicodex** | 控制中心，管理所有 vibe-kanban 实例的生命周期 |
| **Instance Manager** | 实例 CRUD、进程启停、健康检查、自动恢复 |
| **Proxy Router** | 将前端请求路由到正确的 vibe-kanban 实例 |
| **Agent Config Manager** | AI 智能体配置管理、API Key 加密、环境变量注入 |
| **User Manager** | 用户管理、用户与实例绑定、权限控制 |
| **vibe-kanban 实例** | 独立运行的子进程，处理实际的项目和任务管理 |

### 用户与实例绑定模型

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        用户-实例绑定关系 (N:M 多对多)                          │
│                                                                              │
│   ┌─────────┐                           ┌─────────────────┐                  │
│   │  用户 A │ ─────────────────────────▶│  vibe-kanban    │                  │
│   │  ★当前  │ ──┐                       │  实例 1         │                  │
│   └─────────┘   │                       │  Port: 18100    │                  │
│                 │                       │  团队: 开发组    │                  │
│   ┌─────────┐   │                       └─────────────────┘                  │
│   │  用户 B │ ──┼──────────────────────▶        ▲                            │
│   │  ★当前  │   │                               │                            │
│   └─────────┘   │                               │                            │
│                 │                               │                            │
│   ┌─────────┐   │    ┌─────────────────────────┘                            │
│   │  用户 C │ ──┴───▶│                                                       │
│   │         │ ──────▶│  ┌─────────────────┐                                  │
│   │  ★当前  │        └─▶│  vibe-kanban    │                                  │
│   └─────────┘           │  实例 2         │                                  │
│                         │  Port: 18101    │                                  │
│   ┌─────────┐           │  团队: 测试组    │                                  │
│   │  用户 D │ ─────────▶└─────────────────┘                                  │
│   │  ★当前  │                   ▲                                            │
│   └─────────┘                   │                                            │
│                                 │                                            │
│   ┌─────────┐                   │          ┌─────────────────┐               │
│   │  用户 E │ ──────────────────┘          │  vibe-kanban    │               │
│   │  (管理员)│ ───────────────────────────▶│  实例 3         │               │
│   │  ★当前  │ ───────────────────────────▶│  Port: 18102    │               │
│   └─────────┘ ───────────────────────────▶│  团队: 生产组    │               │
│              (管理员可访问所有实例)         └─────────────────┘               │
│                                                                              │
│   图例：                                                                      │
│   ─────▶  表示用户被分配到该实例（可访问）                                     │
│   ★当前   表示用户当前选中的实例（请求将路由到此实例）                          │
│                                                                              │
│   说明：                                                                      │
│   - 管理员创建 vibe-kanban 实例                                              │
│   - 管理员将用户分配到一个或多个实例（N:M 多对多关系）                         │
│   - 用户可以切换当前使用的实例                                                │
│   - 请求自动路由到用户当前选中的实例                                          │
│   - 用户 C 被分配到实例 1 和实例 2，可以在两者之间切换                         │
│   - 管理员（用户 E）通常被分配到所有实例以便管理                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 实例生命周期

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  创建    │────▶│  启动    │────▶│  运行中  │────▶│  停止    │
│ Created  │     │ Starting │     │ Running  │     │ Stopped  │
└──────────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
                      │                │                │
                      │                ▼                │
                      │          ┌──────────┐          │
                      │          │ 健康检查 │          │
                      │          │ 失败     │          │
                      │          └────┬─────┘          │
                      │               │                │
                      │               ▼                │
                      │          ┌──────────┐          │
                      └──────────│ 自动重启 │──────────┘
                                 └──────────┘
```

### 生命周期流程详解

#### 1. 创建实例 (Create)
```
用户请求 → aicodex 分配端口 → 创建数据目录 → 写入数据库 → 返回实例信息
```

#### 2. 启动实例 (Start)
```
用户请求 → 读取实例配置 → 准备环境变量 → 启动子进程 → 等待健康检查 → 更新状态
```

环境变量注入：
```bash
VIBE_KANBAN_DATA_DIR=/data/vibe-instances/{instance-id}
VIBE_KANBAN_DB_PATH=/data/vibe-instances/{instance-id}/db/db.sqlite
VIBE_KANBAN_PORT={allocated-port}
VIBE_KANBAN_HOST=127.0.0.1

# AI 智能体环境变量
ANTHROPIC_API_KEY={decrypted-key}
CLAUDE_CONFIG_DIR=/data/vibe-instances/{instance-id}/ai-agents/claude-code
OPENAI_API_KEY={decrypted-key}
CODEX_CONFIG_HOME=/data/vibe-instances/{instance-id}/ai-agents/codex-cli
# ... 其他智能体
```

#### 3. 运行中监控 (Running)
```
定时器触发 → HTTP 健康检查 → 更新健康状态 → 连续失败则触发自动重启
```

#### 4. 停止实例 (Stop)
```
用户请求 → 发送 SIGTERM → 等待优雅关闭(30s) → 超时则 SIGKILL → 更新状态
```

#### 5. 删除实例 (Delete)
```
用户请求 → 检查实例已停止 → 删除数据目录 → 释放端口 → 删除数据库记录
```

### 请求路由流程

```
┌──────────┐      ┌──────────┐      ┌──────────────┐      ┌──────────────┐
│ 前端请求 │─────▶│ aicodex  │─────▶│ 查找实例     │─────▶│ 检查状态     │
│          │      │ Proxy    │      │ by ID        │      │ Running?     │
└──────────┘      └──────────┘      └──────────────┘      └──────┬───────┘
                                                                  │
                       ┌──────────────────────────────────────────┘
                       │
                       ▼
              ┌────────────────┐      ┌──────────────────┐
              │ 构建目标 URL   │─────▶│ 转发请求         │
              │ 127.0.0.1:port │      │ 返回响应         │
              └────────────────┘      └──────────────────┘
```

示例：
```
GET /api/proxy/projects (带有用户会话 Cookie)
    │
    ▼
aicodex 从会话获取用户 → 查找用户分配的实例 inst-001 → 端口 18100
    │
    ▼
转发到 http://127.0.0.1:18100/api/projects
    │
    ▼
返回 vibe-kanban 响应
```

## Goals / Non-Goals

### Goals
1. 支持 aicodex 管理多个 vibe-kanban 实例
2. 每个实例拥有独立的数据存储目录
3. 每个实例拥有独立的配置（端口、环境变量）
4. 每个实例拥有独立的 AI 智能体配置和凭证
5. 提供实例的生命周期管理（创建、启动、停止、删除）
6. 提供实例健康检查和自动恢复机制

### Non-Goals
1. 不修改 vibe-kanban 核心代码（仅通过环境变量控制）
2. 不实现跨实例的数据共享
3. 不实现实例间的负载均衡
4. 不实现 AI 智能体的细粒度权限控制（仅隔离）

## Vibe-Kanban 实例管理系统

### 系统概述

vibe-kanban 实例管理系统是 aicodex 的核心模块，负责创建、配置和管理多个 vibe-kanban 实例。每个实例使用 UUID v4 作为唯一标识符。

### 实例标识规范

```
实例 ID 格式: UUID v4
示例: 550e8400-e29b-41d4-a716-446655440000

数据目录命名: /data/vibe-instances/{uuid}/
示例: /data/vibe-instances/550e8400-e29b-41d4-a716-446655440000/
```

### 实例管理系统架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Vibe-Kanban 实例管理系统                                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        管理员控制台 (Admin UI)                       │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │ 实例列表    │  │ 创建实例   │  │ 实例详情    │  │ 用户分配  │  │   │
│  │  │             │  │             │  │             │  │           │  │   │
│  │  │ - 状态监控  │  │ - 名称配置 │  │ - 配置编辑  │  │ - 批量分配│  │   │
│  │  │ - 批量操作  │  │ - AI 配置  │  │ - 用户管理  │  │ - 移除用户│  │   │
│  │  │ - 搜索过滤  │  │ - 资源限制 │  │ - 日志查看  │  │           │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      实例管理 API (/api/instances)                   │   │
│  │                                                                     │   │
│  │  POST   /                    创建实例 (返回 UUID)                   │   │
│  │  GET    /                    列出所有实例                           │   │
│  │  GET    /{uuid}              获取实例详情                           │   │
│  │  PUT    /{uuid}              更新实例配置                           │   │
│  │  DELETE /{uuid}              删除实例                               │   │
│  │  POST   /{uuid}/start        启动实例                               │   │
│  │  POST   /{uuid}/stop         停止实例                               │   │
│  │  POST   /{uuid}/restart      重启实例                               │   │
│  │  GET    /{uuid}/health       健康检查                               │   │
│  │  GET    /{uuid}/logs         获取实例日志                           │   │
│  │  GET    /{uuid}/users        获取实例用户列表                       │   │
│  │  GET    /{uuid}/stats        获取实例统计信息                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       InstanceManager (Rust)                         │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │InstanceRepo │  │ProcessMgr   │  │PortAllocator│  │HealthCheck│  │   │
│  │  │             │  │             │  │             │  │           │  │   │
│  │  │ - CRUD      │  │ - spawn     │  │ - allocate  │  │ - probe   │  │   │
│  │  │ - query     │  │ - kill      │  │ - release   │  │ - recover │  │   │
│  │  │ - persist   │  │ - monitor   │  │ - check     │  │ - alert   │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 核心数据模型

```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Vibe-Kanban 实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeInstance {
    /// 唯一标识符 (UUID v4)
    pub id: Uuid,

    /// 实例名称 (用于显示)
    pub name: String,

    /// 实例描述
    pub description: Option<String>,

    /// 分配的端口
    pub port: u16,

    /// 数据目录路径
    pub data_dir: String,

    /// 当前状态
    pub status: InstanceStatus,

    /// 健康状态
    pub health_status: HealthStatus,

    /// 是否自动启动 (用户登录时)
    pub auto_start: bool,

    /// 最大用户数限制 (0 = 无限制)
    pub max_users: u32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 最后健康检查时间
    pub last_health_check: Option<DateTime<Utc>>,
}

/// 实例状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    /// 已停止
    Stopped,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
    /// 错误状态
    Error,
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// 未知
    Unknown,
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
}

/// 创建实例请求
#[derive(Debug, Deserialize)]
pub struct CreateInstanceRequest {
    /// 实例名称
    pub name: String,

    /// 实例描述
    pub description: Option<String>,

    /// 是否自动启动
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,

    /// 最大用户数
    #[serde(default)]
    pub max_users: u32,

    /// AI 智能体配置
    pub ai_agents: Option<Vec<AiAgentConfig>>,
}

fn default_auto_start() -> bool { true }

/// AI 智能体配置
#[derive(Debug, Deserialize)]
pub struct AiAgentConfig {
    pub agent_type: String,  // "claude-code", "codex-cli", "gemini-cli", "opencode"
    pub api_key: Option<String>,
    pub config: Option<serde_json::Value>,
}
```

### InstanceManager 实现

```rust
use uuid::Uuid;
use tokio::process::{Child, Command};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InstanceManager {
    /// 实例数据库仓库
    repo: InstanceRepository,

    /// 运行中的进程 (UUID -> Child)
    processes: Arc<RwLock<HashMap<Uuid, Child>>>,

    /// 端口分配器
    port_allocator: PortAllocator,

    /// 数据根目录
    data_root: PathBuf,

    /// vibe-kanban 可执行文件路径
    vibe_kanban_bin: PathBuf,
}

impl InstanceManager {
    /// 创建新实例
    pub async fn create(&self, req: CreateInstanceRequest) -> Result<VibeInstance> {
        // 1. 生成 UUID
        let id = Uuid::new_v4();

        // 2. 分配端口
        let port = self.port_allocator.allocate().await?;

        // 3. 创建数据目录
        let data_dir = self.data_root.join(id.to_string());
        self.create_data_directories(&data_dir).await?;

        // 4. 创建实例记录
        let instance = VibeInstance {
            id,
            name: req.name,
            description: req.description,
            port,
            data_dir: data_dir.to_string_lossy().to_string(),
            status: InstanceStatus::Stopped,
            health_status: HealthStatus::Unknown,
            auto_start: req.auto_start,
            max_users: req.max_users,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_health_check: None,
        };

        // 5. 保存到数据库
        self.repo.insert(&instance).await?;

        // 6. 配置 AI 智能体
        if let Some(agents) = req.ai_agents {
            for agent in agents {
                self.configure_ai_agent(&id, agent).await?;
            }
        }

        tracing::info!(instance_id = %id, name = %instance.name, "Instance created");

        Ok(instance)
    }

    /// 启动实例
    pub async fn start(&self, id: &Uuid) -> Result<()> {
        let mut instance = self.repo.get(id).await?
            .ok_or(Error::InstanceNotFound)?;

        if instance.status == InstanceStatus::Running {
            return Ok(()); // 已经在运行
        }

        // 更新状态为启动中
        instance.status = InstanceStatus::Starting;
        self.repo.update(&instance).await?;

        // 准备环境变量
        let env_vars = self.prepare_environment(&instance).await?;

        // 启动进程
        let child = Command::new(&self.vibe_kanban_bin)
            .envs(env_vars)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // 保存进程句柄
        self.processes.write().await.insert(*id, child);

        // 等待健康检查通过
        self.wait_for_healthy(id, Duration::from_secs(30)).await?;

        // 更新状态为运行中
        instance.status = InstanceStatus::Running;
        instance.health_status = HealthStatus::Healthy;
        instance.updated_at = Utc::now();
        self.repo.update(&instance).await?;

        tracing::info!(instance_id = %id, "Instance started");

        Ok(())
    }

    /// 停止实例
    pub async fn stop(&self, id: &Uuid) -> Result<()> {
        let mut instance = self.repo.get(id).await?
            .ok_or(Error::InstanceNotFound)?;

        if instance.status == InstanceStatus::Stopped {
            return Ok(()); // 已经停止
        }

        // 更新状态为停止中
        instance.status = InstanceStatus::Stopping;
        self.repo.update(&instance).await?;

        // 获取进程句柄
        let mut processes = self.processes.write().await;
        if let Some(mut child) = processes.remove(id) {
            // 发送 SIGTERM
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Some(pid) = child.id() {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
            }

            // 等待优雅关闭
            match tokio::time::timeout(
                Duration::from_secs(30),
                child.wait()
            ).await {
                Ok(_) => {},
                Err(_) => {
                    // 超时，强制终止
                    let _ = child.kill().await;
                }
            }
        }

        // 更新状态为已停止
        instance.status = InstanceStatus::Stopped;
        instance.health_status = HealthStatus::Unknown;
        instance.updated_at = Utc::now();
        self.repo.update(&instance).await?;

        tracing::info!(instance_id = %id, "Instance stopped");

        Ok(())
    }

    /// 删除实例
    pub async fn delete(&self, id: &Uuid) -> Result<()> {
        let instance = self.repo.get(id).await?
            .ok_or(Error::InstanceNotFound)?;

        // 检查实例是否已停止
        if instance.status != InstanceStatus::Stopped {
            return Err(Error::InstanceMustBeStopped);
        }

        // 检查是否有用户分配
        let user_count = self.repo.count_users(id).await?;
        if user_count > 0 {
            return Err(Error::InstanceHasUsers(user_count));
        }

        // 删除数据目录
        let data_dir = PathBuf::from(&instance.data_dir);
        if data_dir.exists() {
            tokio::fs::remove_dir_all(&data_dir).await?;
        }

        // 释放端口
        self.port_allocator.release(instance.port).await;

        // 从数据库删除
        self.repo.delete(id).await?;

        tracing::info!(instance_id = %id, "Instance deleted");

        Ok(())
    }

    /// 列出所有实例
    pub async fn list(&self) -> Result<Vec<VibeInstance>> {
        self.repo.list().await
    }

    /// 获取实例详情
    pub async fn get(&self, id: &Uuid) -> Result<Option<VibeInstance>> {
        self.repo.get(id).await
    }

    /// 根据用户获取实例
    pub async fn get_by_user(&self, user_id: &Uuid) -> Result<Option<VibeInstance>> {
        self.repo.get_by_user(user_id).await
    }

    /// 准备环境变量
    async fn prepare_environment(&self, instance: &VibeInstance) -> Result<Vec<(String, String)>> {
        let mut env = vec![
            ("VIBE_KANBAN_DATA_DIR".to_string(), instance.data_dir.clone()),
            ("VIBE_KANBAN_DB_PATH".to_string(), format!("{}/db/db.sqlite", instance.data_dir)),
            ("VIBE_KANBAN_PORT".to_string(), instance.port.to_string()),
            ("VIBE_KANBAN_HOST".to_string(), "127.0.0.1".to_string()),
        ];

        // 加载 AI 智能体配置
        let agents = self.repo.get_ai_agents(&instance.id).await?;
        for agent in agents {
            if agent.is_enabled {
                env.extend(self.get_agent_env_vars(&instance.id, &agent).await?);
            }
        }

        Ok(env)
    }

    /// 创建数据目录结构
    async fn create_data_directories(&self, data_dir: &Path) -> Result<()> {
        let dirs = ["db", "config", "worktrees", "logs", "ai-agents/claude-code",
                    "ai-agents/codex-cli", "ai-agents/gemini-cli", "ai-agents/opencode"];

        for dir in &dirs {
            tokio::fs::create_dir_all(data_dir.join(dir)).await?;
        }

        Ok(())
    }
}
```

### 管理员 UI 设计

#### 实例列表页面 (`/admin/instances`)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  Vibe-Kanban 实例管理                                      [+ 创建实例]      │
├──────────────────────────────────────────────────────────────────────────────┤
│  搜索: [________________]  状态: [全部 ▼]  排序: [创建时间 ▼]               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ ● 开发组实例                                             [运行中]      │ │
│  │ ID: 550e8400-e29b-41d4-a716-446655440000                              │ │
│  │ 端口: 18100  |  用户: 5/10  |  健康: ✓                                │ │
│  │ 创建于: 2026-01-15 10:30                                              │ │
│  │                                                                        │ │
│  │ [启动] [停止] [重启] [配置] [用户] [日志] [删除]                      │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ ○ 测试组实例                                             [已停止]      │ │
│  │ ID: 7c9e6679-7425-40de-944b-e07fc1f90ae7                              │ │
│  │ 端口: 18101  |  用户: 3/5   |  健康: -                                │ │
│  │ 创建于: 2026-01-16 14:20                                              │ │
│  │                                                                        │ │
│  │ [启动] [停止] [重启] [配置] [用户] [日志] [删除]                      │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ ⚠ 生产组实例                                             [错误]        │ │
│  │ ID: b3d4c5e6-f7a8-4b9c-0d1e-2f3a4b5c6d7e                              │ │
│  │ 端口: 18102  |  用户: 8/∞   |  健康: ✗                                │ │
│  │ 错误: Process exited with code 1                                       │ │
│  │                                                                        │ │
│  │ [启动] [停止] [重启] [配置] [用户] [日志] [删除]                      │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  显示 1-3 / 共 3 个实例                                    [< 1 2 3 >]     │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### 创建实例对话框

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  创建新实例                                                          [×]    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  基本信息                                                                    │
│  ─────────────────────────────────────────────────────                      │
│  实例名称 *     [________________________]                                   │
│  描述           [________________________]                                   │
│                                                                              │
│  配置选项                                                                    │
│  ─────────────────────────────────────────────────────                      │
│  最大用户数     [10_______]  (0 = 无限制)                                   │
│  自动启动       [✓] 用户登录时自动启动实例                                  │
│                                                                              │
│  AI 智能体配置                                                               │
│  ─────────────────────────────────────────────────────                      │
│  [✓] Claude Code                                                            │
│      API Key: [sk-ant-*************************]                            │
│      模型:    [claude-sonnet-4-20250514 ▼]                                  │
│                                                                              │
│  [✓] Codex CLI                                                              │
│      API Key: [sk-****************************]                             │
│                                                                              │
│  [ ] Gemini CLI                                                             │
│  [ ] OpenCode                                                               │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                              [取消]  [创建并启动]  [仅创建]  │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### 实例详情页面 (`/admin/instances/{uuid}`)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  ← 返回列表    开发组实例                           [启动] [停止] [重启]    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─ 基本信息 ────────────────────────────────────────────────────────────┐  │
│  │ ID:        550e8400-e29b-41d4-a716-446655440000                       │  │
│  │ 名称:      开发组实例                                     [编辑]      │  │
│  │ 描述:      用于开发团队的工作空间                         [编辑]      │  │
│  │ 状态:      ● 运行中                                                   │  │
│  │ 健康:      ✓ 健康 (最后检查: 30秒前)                                  │  │
│  │ 端口:      18100                                                      │  │
│  │ 数据目录:  /data/vibe-instances/550e8400-e29b-41d4-a716-446655440000  │  │
│  │ 创建时间:  2026-01-15 10:30:00                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ 用户管理 (5/10) ─────────────────────────────────────────────────────┐  │
│  │                                                        [+ 分配用户]    │  │
│  │  用户名          邮箱                    分配时间           操作       │  │
│  │  ───────────────────────────────────────────────────────────────────  │  │
│  │  alice           alice@example.com      2026-01-15 10:35   [移除]     │  │
│  │  bob             bob@example.com        2026-01-15 11:20   [移除]     │  │
│  │  charlie         charlie@example.com    2026-01-16 09:00   [移除]     │  │
│  │  david           david@example.com      2026-01-17 14:30   [移除]     │  │
│  │  eve             eve@example.com        2026-01-18 16:45   [移除]     │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ AI 智能体配置 ───────────────────────────────────────────────────────┐  │
│  │                                                                       │  │
│  │  Claude Code    [✓ 已启用]   API Key: sk-ant-***...   [测试] [配置]  │  │
│  │  Codex CLI      [✓ 已启用]   API Key: sk-***...       [测试] [配置]  │  │
│  │  Gemini CLI     [○ 未配置]                            [配置]         │  │
│  │  OpenCode       [○ 未配置]                            [配置]         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ 使用统计 (最近7天) ──────────────────────────────────────────────────┐  │
│  │                                                                       │  │
│  │  日期        Claude Code      Codex CLI       总请求      Token消耗  │  │
│  │  ─────────────────────────────────────────────────────────────────── │  │
│  │  2026-01-21      152              38            190         45,230   │  │
│  │  2026-01-20      143              42            185         42,100   │  │
│  │  2026-01-19      128              35            163         38,500   │  │
│  │  ...                                                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌─ 实例日志 ────────────────────────────────────────────────────────────┐  │
│  │  [实时刷新 ✓]                                          [下载完整日志] │  │
│  │  ─────────────────────────────────────────────────────────────────── │  │
│  │  2026-01-21 14:30:15 [INFO]  Server started on port 18100             │  │
│  │  2026-01-21 14:30:16 [INFO]  Database connected                       │  │
│  │  2026-01-21 14:30:17 [INFO]  Ready to accept connections              │  │
│  │  2026-01-21 14:35:22 [INFO]  User alice logged in                     │  │
│  │  ...                                                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Decisions

### 1. 实例数据目录结构

**Decision**: 使用统一的数据根目录，每个实例在子目录中存储数据

```
/data/vibe-instances/
├── {instance-id}/
│   ├── db/
│   │   └── db.sqlite           # 实例数据库
│   ├── config/
│   │   └── instance.toml       # 实例配置
│   ├── worktrees/              # Git 工作树
│   ├── logs/                   # 实例日志
│   └── ai-agents/              # AI 智能体配置
│       ├── claude-code/
│       │   └── settings.json
│       ├── codex-cli/
│       │   └── config.yaml
│       ├── gemini-cli/
│       │   └── config.json
│       └── opencode/
│           └── config.toml
```

**Alternatives considered**:
- 使用 Docker 卷：增加部署复杂性，不利于调试
- 使用数据库存储所有数据：性能问题，不符合 vibe-kanban 设计

**Rationale**: 文件系统隔离简单可靠，易于备份和迁移

### 2. 实例进程管理

**Decision**: 使用 Rust 的 `tokio::process` 管理子进程，每个实例作为独立进程运行

```rust
use uuid::Uuid;

pub struct VibeInstance {
    pub id: Uuid,  // UUID v4
    pub name: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub process: Option<Child>,
    pub status: InstanceStatus,
}

pub enum InstanceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}
```

**Environment Variables for Instance**:
```bash
VIBE_KANBAN_DATA_DIR=/data/vibe-instances/{instance-id}
VIBE_KANBAN_DB_PATH=/data/vibe-instances/{instance-id}/db/db.sqlite
VIBE_KANBAN_PORT={allocated-port}
VIBE_KANBAN_HOST=127.0.0.1
```

**Alternatives considered**:
- 使用 systemd 管理：需要 root 权限，不够灵活
- 使用 Docker：增加依赖，不利于开发调试

**Rationale**: 直接进程管理提供最大灵活性和控制力

### 3. 端口分配策略

**Decision**: 使用动态端口分配，从配置的端口范围中分配

```rust
pub struct PortAllocator {
    base_port: u16,      // 默认 18100
    max_instances: u16,  // 默认 100
    allocated: HashSet<u16>,
}
```

**Port Range**: `18100-18199` (可配置)

**Alternatives considered**:
- 固定端口映射：不够灵活，手动管理困难
- 随机端口：调试困难，不可预测

**Rationale**: 范围分配平衡了可预测性和灵活性

### 4. AI 智能体隔离架构

**Decision**: 每个实例拥有独立的 AI 智能体配置目录，通过环境变量注入

#### 4.1 Claude Code 隔离

```bash
# 实例环境变量
ANTHROPIC_API_KEY={instance-specific-key}
CLAUDE_CONFIG_DIR=/data/vibe-instances/{instance-id}/ai-agents/claude-code
```

配置文件结构:
```json
// /data/vibe-instances/{instance-id}/ai-agents/claude-code/settings.json
{
  "apiKey": "sk-ant-xxx",
  "model": "claude-sonnet-4-20250514",
  "maxTokens": 8192,
  "customInstructions": "..."
}
```

#### 4.2 Codex CLI 隔离

```bash
OPENAI_API_KEY={instance-specific-key}
CODEX_CONFIG_HOME=/data/vibe-instances/{instance-id}/ai-agents/codex-cli
```

#### 4.3 Gemini CLI 隔离

```bash
GOOGLE_API_KEY={instance-specific-key}
GEMINI_CONFIG_DIR=/data/vibe-instances/{instance-id}/ai-agents/gemini-cli
```

#### 4.4 OpenCode 隔离

```bash
OPENCODE_CONFIG_DIR=/data/vibe-instances/{instance-id}/ai-agents/opencode
```

**Alternatives considered**:
- 共享凭证池：安全风险，无法追踪使用
- 外部密钥管理服务：增加复杂性

**Rationale**: 基于环境变量的隔离简单有效，符合各 CLI 工具的设计

### 5. 数据库 Schema

**Decision**: 在 aicodex 数据库中添加用户管理和实例管理表

```sql
-- 用户表
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    role TEXT NOT NULL DEFAULT 'user',  -- 'admin', 'user'
    current_instance_id TEXT REFERENCES vibe_instances(id),  -- 当前选中的实例（可为空）
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login_at TEXT
);

-- 实例表
CREATE TABLE vibe_instances (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    port INTEGER NOT NULL UNIQUE,
    data_dir TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'stopped',
    auto_start BOOLEAN NOT NULL DEFAULT 1,  -- 有用户登录时自动启动
    max_users INTEGER DEFAULT 0,  -- 最大用户数限制，0表示不限制
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_health_check TEXT,
    health_status TEXT DEFAULT 'unknown'
);

-- 用户-实例分配表（多对多关系）
CREATE TABLE user_instance_assignments (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    instance_id TEXT NOT NULL REFERENCES vibe_instances(id) ON DELETE CASCADE,
    assigned_by TEXT REFERENCES users(id),  -- 分配者（管理员）
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, instance_id)
);

-- 用户会话表
CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip_address TEXT,
    user_agent TEXT
);

-- AI 智能体配置表（每个实例独立配置）
CREATE TABLE instance_ai_agents (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES vibe_instances(id) ON DELETE CASCADE,
    agent_type TEXT NOT NULL, -- 'claude-code', 'codex-cli', 'gemini-cli', 'opencode'
    is_enabled BOOLEAN NOT NULL DEFAULT 1,
    api_key_encrypted TEXT,  -- 加密存储
    config_json TEXT,        -- 其他配置
    rate_limit_rpm INTEGER,  -- 每分钟请求限制
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(instance_id, agent_type)
);

-- 实例使用统计表
CREATE TABLE instance_usage_stats (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES vibe_instances(id) ON DELETE CASCADE,
    agent_type TEXT NOT NULL,
    date TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    token_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(instance_id, agent_type, date)
);

-- 索引
CREATE INDEX idx_users_current_instance ON users(current_instance_id);
CREATE INDEX idx_assignments_user_id ON user_instance_assignments(user_id);
CREATE INDEX idx_assignments_instance_id ON user_instance_assignments(instance_id);
CREATE INDEX idx_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON user_sessions(expires_at);
```

### 6. 用户认证与实例分配

**Decision**: 管理员分配用户到多个实例，用户可切换当前实例

#### 6.1 管理员分配用户流程

```
管理员创建实例 → 管理员创建用户 → 管理员将用户分配到一个或多个实例
```

```rust
use uuid::Uuid;

/// 用户-实例分配记录
pub struct UserInstanceAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub instance_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
}

// 管理员将用户分配到实例（支持多实例）
pub async fn assign_user_to_instance(
    admin: &User,
    user_id: &Uuid,
    instance_id: &Uuid,
) -> Result<()> {
    // 1. 验证管理员权限
    if admin.role != "admin" {
        return Err(Error::Forbidden);
    }

    // 2. 验证实例存在
    let instance = InstanceManager::get(instance_id).await?;

    // 3. 检查实例用户数限制
    if instance.max_users > 0 {
        let current_users = count_users_in_instance(instance_id).await?;
        if current_users >= instance.max_users {
            return Err(Error::InstanceFull);
        }
    }

    // 4. 检查是否已分配
    if is_user_assigned(user_id, instance_id).await? {
        return Ok(()); // 已分配，幂等操作
    }

    // 5. 创建分配记录
    let assignment = UserInstanceAssignment {
        id: Uuid::new_v4(),
        user_id: *user_id,
        instance_id: *instance_id,
        assigned_by: admin.id,
        assigned_at: Utc::now(),
    };
    insert_assignment(&assignment).await?;

    // 6. 如果用户没有当前实例，设置为此实例
    let user = get_user(user_id).await?;
    if user.current_instance_id.is_none() {
        set_current_instance(user_id, instance_id).await?;
    }

    Ok(())
}

// 取消用户的实例分配
pub async fn unassign_user_from_instance(
    admin: &User,
    user_id: &Uuid,
    instance_id: &Uuid,
) -> Result<()> {
    // 1. 验证管理员权限
    if admin.role != "admin" {
        return Err(Error::Forbidden);
    }

    // 2. 删除分配记录
    delete_assignment(user_id, instance_id).await?;

    // 3. 如果这是用户当前实例，切换到其他已分配实例
    let user = get_user(user_id).await?;
    if user.current_instance_id == Some(*instance_id) {
        let other_instances = get_user_instances(user_id).await?;
        if let Some(first) = other_instances.first() {
            set_current_instance(user_id, &first.id).await?;
        } else {
            clear_current_instance(user_id).await?;
        }
    }

    Ok(())
}
```

#### 6.2 用户登录流程

```
用户登录 → 验证凭证 → 检查是否已分配实例 → 自动启动当前实例(如未运行) → 返回会话Token和实例列表
```

```rust
pub async fn login(username: &str, password: &str) -> Result<LoginResponse> {
    // 1. 验证用户
    let user = verify_user(username, password).await?;

    // 2. 获取用户分配的所有实例
    let instances = get_user_instances(&user.id).await?;

    if instances.is_empty() {
        return Err(Error::NoInstanceAssigned("请联系管理员分配工作空间"));
    }

    // 3. 确定当前实例
    let current_instance_id = user.current_instance_id
        .unwrap_or(instances[0].id);  // 默认使用第一个分配的实例

    // 4. 获取当前实例
    let current_instance = InstanceManager::get(&current_instance_id).await?;

    // 5. 如果当前实例未运行且配置了自动启动，则启动
    if current_instance.status != Running && current_instance.auto_start {
        InstanceManager::start(&current_instance.id).await?;
    }

    // 6. 创建会话
    let session = create_session(&user.id).await?;

    Ok(LoginResponse {
        token: session.token,
        user: user.into(),
        instances: instances.into_iter().map(|i| i.into()).collect(),
        current_instance_id,
    })
}
```

#### 6.3 用户切换实例

```rust
/// 用户切换当前实例
pub async fn switch_instance(
    user_id: &Uuid,
    instance_id: &Uuid,
) -> Result<InstanceInfo> {
    // 1. 验证用户有权访问该实例
    if !is_user_assigned(user_id, instance_id).await? {
        return Err(Error::InstanceNotAssigned);
    }

    // 2. 获取实例
    let instance = InstanceManager::get(instance_id).await?
        .ok_or(Error::InstanceNotFound)?;

    // 3. 如果实例未运行且配置了自动启动，则启动
    if instance.status != InstanceStatus::Running && instance.auto_start {
        InstanceManager::start(instance_id).await?;
    }

    // 4. 更新用户当前实例
    set_current_instance(user_id, instance_id).await?;

    Ok(instance.into())
}
```

#### 6.4 请求路由（基于用户当前实例）

```rust
// 从会话自动路由到用户当前选中的实例
// 前端不需要显式传递 instance-id

async fn proxy_handler(
    State(state): State<AppState>,
    session: Session,  // 从 Cookie/Header 提取
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    // 1. 从会话获取用户
    let user = get_user(&session.user_id).await?;

    // 2. 检查用户当前实例
    let instance_id = user.current_instance_id
        .ok_or(Error::NoInstanceSelected)?;

    // 3. 验证用户有权访问该实例
    if !is_user_assigned(&user.id, &instance_id).await? {
        return Err(Error::InstanceNotAssigned);
    }

    // 4. 查找实例
    let instance = state.instance_manager.get(&instance_id)?;

    // 5. 检查实例状态
    if instance.status != InstanceStatus::Running {
        if instance.auto_start {
            state.instance_manager.start(&instance.id).await?;
        } else {
            return Err(Error::InstanceNotRunning);
        }
    }

    // 6. 代理请求
    let target_url = format!("http://127.0.0.1:{}/api/{}", instance.port, path);
    proxy_request(target_url, req).await
}
```

#### 6.5 实例切换流程图

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              用户多实例切换流程                                   │
│                                                                                 │
│  ┌─────────────────┐                                                            │
│  │   用户登录       │                                                            │
│  │   POST /login   │                                                            │
│  └────────┬────────┘                                                            │
│           │                                                                     │
│           ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │  返回:                                                                   │   │
│  │  - token: "jwt_xxx"                                                     │   │
│  │  - instances: [                                                         │   │
│  │      {id: "uuid-1", name: "开发组", status: "running"},                 │   │
│  │      {id: "uuid-2", name: "测试组", status: "stopped"}                  │   │
│  │    ]                                                                    │   │
│  │  - current_instance_id: "uuid-1"                                        │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│           │                                                                     │
│           ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                            前端实例选择器                                 │   │
│  │  ┌───────────────────────────────────────────────────────────────────┐  │   │
│  │  │  当前实例: [开发组 ▼]                                              │  │   │
│  │  │            ┌──────────────────┐                                   │  │   │
│  │  │            │ ● 开发组 (运行中) │ ← 当前                            │  │   │
│  │  │            │ ○ 测试组 (已停止) │                                   │  │   │
│  │  │            └──────────────────┘                                   │  │   │
│  │  └───────────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│           │                                                                     │
│           │ 用户选择切换到 "测试组"                                              │
│           ▼                                                                     │
│  ┌─────────────────┐                      ┌─────────────────────────────────┐  │
│  │  PUT /api/my-   │                      │  后端处理:                       │  │
│  │  instances/     │ ────────────────────▶│  1. 验证用户有权限               │  │
│  │  current        │                      │  2. 自动启动实例(如已停止)       │  │
│  │  {id: "uuid-2"} │                      │  3. 更新 current_instance_id    │  │
│  └─────────────────┘                      │  4. 返回实例信息                 │  │
│           │                               └─────────────────────────────────┘  │
│           ▼                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │  后续所有 /api/proxy/* 请求自动路由到 "测试组" 实例 (uuid-2)              │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 7. API 设计

**Decision**: RESTful API 设计

```
# 用户认证
POST   /api/auth/register              # 用户注册（需管理员后续分配实例）
POST   /api/auth/login                 # 用户登录（返回用户信息、实例列表、当前实例）
POST   /api/auth/logout                # 用户登出
GET    /api/auth/me                    # 获取当前用户信息（含实例列表）
PUT    /api/auth/password              # 修改密码

# 用户管理（管理员）
GET    /api/users                      # 列出所有用户
POST   /api/users                      # 创建用户
GET    /api/users/{id}                 # 获取用户详情（含分配的实例列表）
PUT    /api/users/{id}                 # 更新用户信息
DELETE /api/users/{id}                 # 删除用户
PUT    /api/users/{id}/activate        # 激活/停用用户
GET    /api/users/{id}/instances       # 获取用户分配的所有实例
POST   /api/users/{id}/instances       # 分配用户到实例（管理员）
DELETE /api/users/{id}/instances/{instance_id}  # 取消用户实例分配（管理员）

# 实例管理（管理员）
POST   /api/instances                  # 创建实例
GET    /api/instances                  # 列出所有实例
GET    /api/instances/{id}             # 获取实例详情（含用户列表）
PUT    /api/instances/{id}             # 更新实例配置
DELETE /api/instances/{id}             # 删除实例
POST   /api/instances/{id}/start       # 启动实例
POST   /api/instances/{id}/stop        # 停止实例
POST   /api/instances/{id}/restart     # 重启实例
GET    /api/instances/{id}/health      # 健康检查
GET    /api/instances/{id}/users       # 获取实例的用户列表
GET    /api/instances/{id}/logs        # 获取实例日志
GET    /api/instances/{id}/stats       # 获取实例使用统计

# 当前用户的实例管理
GET    /api/my-instances               # 获取当前用户分配的所有实例列表
GET    /api/my-instances/current       # 获取当前选中的实例
PUT    /api/my-instances/current       # 切换当前实例 ← 核心功能
GET    /api/my-instances/current/health  # 当前实例健康检查

# AI 智能体配置（管理员，每个实例独立）
GET    /api/instances/{id}/agents              # 列出实例的 AI 智能体配置
PUT    /api/instances/{id}/agents/{type}       # 配置 AI 智能体
POST   /api/instances/{id}/agents/{type}/test  # 测试 AI 智能体连接

# 代理路由（基于会话自动路由到用户当前选中的实例）
ANY    /api/proxy/*                    # 自动路由到当前用户选中的实例
```

### 8. 代理路由修改

**Decision**: 修改现有代理路由，支持多实例

```rust
// 新的代理路由格式
// /api/proxy/{instance-id}/{vibe-kanban-path}
// 例如: /api/proxy/inst-001/projects

async fn proxy_handler(
    State(state): State<AppState>,
    Path((instance_id, path)): Path<(String, String)>,
    req: Request<Body>,
) -> impl IntoResponse {
    // 1. 查找实例
    let instance = state.instance_manager.get(&instance_id)?;

    // 2. 检查实例状态
    if instance.status != InstanceStatus::Running {
        return Err(ApiError::InstanceNotRunning);
    }

    // 3. 构建目标 URL
    let target_url = format!("http://127.0.0.1:{}/api/{}", instance.port, path);

    // 4. 代理请求
    proxy_request(target_url, req).await
}
```

**向后兼容**: 保留 `/api/proxy/*` 路由用于默认实例

## Risks / Trade-offs

### Risk 1: 进程管理复杂性
- **风险**: 子进程可能意外退出，需要监控和恢复
- **缓解**: 实现健康检查机制，自动重启失败的实例

### Risk 2: 端口冲突
- **风险**: 分配的端口可能被其他服务占用
- **缓解**: 启动前检查端口可用性，失败时尝试其他端口

### Risk 3: 资源消耗
- **风险**: 每个实例消耗内存和 CPU
- **缓解**: 实现资源限制和监控，提供实例数量上限配置

### Risk 4: API Key 安全
- **风险**: AI 智能体 API Key 存储安全
- **缓解**: 使用加密存储，密钥派生自主密钥

### Trade-off: 复杂性 vs 灵活性
- 选择增加架构复杂性以换取多实例支持的灵活性

## Migration Plan

### Phase 1: 数据库迁移
1. 添加新表 `vibe_instances`, `instance_ai_agents`, `instance_usage_stats`
2. 迁移现有配置到默认实例

### Phase 2: 代理层升级
1. 实现实例管理服务
2. 修改代理路由支持多实例
3. 保持向后兼容

### Phase 3: 前端升级
1. 添加实例管理 UI
2. 添加 AI 智能体配置 UI
3. 修改项目选择器支持实例切换

### Rollback Plan
1. 数据库迁移使用可逆迁移
2. 保留旧的单实例代理路由
3. 配置开关控制新功能启用

## Open Questions

1. **实例资源限制**: 是否需要限制每个实例的 CPU/内存使用？
2. **日志聚合**: 是否需要将所有实例日志聚合到统一位置？
3. **备份策略**: 实例数据如何备份和恢复？
4. **审计日志**: 是否需要记录管理员操作的审计日志？

## Design Decisions - Security

### 密码哈希算法

**Decision**: 使用 Argon2id 进行密码哈希

```rust
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

// 哈希密码
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

// 验证密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
```

**Rationale**: Argon2id 是 2015 年密码哈希竞赛的获胜者，提供最佳的抗 GPU/ASIC 攻击能力。

### 会话管理

**Decision**: JWT Token + 数据库会话双重验证

- **会话有效期**: 默认 24 小时，可配置
- **Token 类型**: JWT (JSON Web Token)
- **刷新机制**: 用户活动时自动延长会话

```rust
pub struct SessionConfig {
    /// 会话有效期（秒），默认 24 小时
    pub session_ttl_secs: u64,  // default: 86400

    /// 会话刷新阈值（秒），剩余时间少于此值时刷新
    pub refresh_threshold_secs: u64,  // default: 3600 (1小时)

    /// 最大并发会话数（每用户）
    pub max_sessions_per_user: u32,  // default: 5
}
```

**Rationale**: 24 小时是用户友好和安全性的平衡点，双重验证允许服务端主动使会话失效。