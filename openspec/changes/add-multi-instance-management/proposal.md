# Change: Add Multi-Instance Vibe-Kanban Management

## Why

当前架构中，aicodex 代理单个 vibe-kanban 实例，无法支持多用户/多团队场景。在服务器部署环境下，需要为每个用户或团队运行独立的 vibe-kanban 实例，实现数据隔离、配置隔离和 AI 智能体隔离。

## What Changes

### 核心功能
- **ADDED** 实例注册与生命周期管理 - aicodex 管理多个 vibe-kanban 实例的启动、停止、健康检查
- **ADDED** 数据目录隔离 - 每个实例使用独立的 SQLite 数据库和工作目录
- **ADDED** 配置隔离 - 每个实例拥有独立的端口、环境变量和配置文件
- **ADDED** AI 智能体隔离 - 每个实例的 AI 智能体（Claude Code、Codex CLI、Gemini CLI、OpenCode）独立配置和运行

### 数据库变更
- **ADDED** `users` 表存储用户信息
- **ADDED** `user_sessions` 表存储用户会话
- **ADDED** `vibe_instances` 表存储实例元数据
- **ADDED** `user_instance_assignments` 表存储用户-实例多对多关系
- **ADDED** `instance_ai_agents` 表存储每个实例的 AI 智能体配置
- **ADDED** `instance_usage_stats` 表存储使用统计

### API 变更
- **ADDED** `/api/auth/*` 用户认证 API
- **ADDED** `/api/users` 用户管理 API
- **ADDED** `/api/instances` 实例管理 API
- **ADDED** `/api/my-instances` 用户实例列表和切换 API
- **MODIFIED** `/api/proxy/*` 基于用户当前选中的实例自动路由

## Impact

- Affected specs: `instance-management`, `ai-agent-isolation` (new)
- Affected code:
  - `aicodex/src/` - 实例管理服务、路由、数据库模型
  - `aicodex/migrations/` - 新数据库迁移
  - `aicodex-web/src/` - 实例管理 UI
- **BREAKING**: 现有单实例配置需要迁移到新的多实例架构
