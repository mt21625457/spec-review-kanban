# Change: 实现完整的设置页面（参照 Vibe-Kanban）

## Why

当前 AICodex-Web 的设置页面功能简陋且 UI 丑陋，仅包含 Gitea 配置、审核配置和队列配置三个简单卡片。需要参照 Vibe-Kanban 实现完整的设置系统，提供与其功能对等的配置能力和美观的用户界面。

## What Changes

### 架构说明

**重要**：AICodex-Web 通过代理 API (`/api/proxy/*`) 访问 Vibe-Kanban 的数据。设置相关的数据（项目、仓库、标签、用户配置、代理配置、MCP 配置）全部存储在 Vibe-Kanban 中，AICodex 不需要重复存储这些数据。

| 数据类型 | 数据源 | API 路径 |
|---------|--------|---------|
| 用户配置（主题、语言、编辑器等） | Vibe-Kanban | `/api/proxy/info`, `/api/proxy/config` |
| 项目管理 | Vibe-Kanban | `/api/proxy/projects/*` |
| 仓库管理 | Vibe-Kanban | `/api/proxy/repos/*` |
| 标签管理 | Vibe-Kanban | `/api/proxy/tags/*` |
| 代理配置 | Vibe-Kanban | `/api/proxy/profiles` |
| MCP 配置 | Vibe-Kanban | `/api/proxy/mcp-config` |
| Gitea/审核/队列配置 | AICodex | `/api/config/*` |

### 前端变更

#### 布局重构
- 将现有单页面设置改为左侧导航 + 右侧内容的多页面布局
- 添加 ESC 键关闭设置页面快捷键
- 实现草稿管理和撤销/保存功能
- 实现离开页面未保存警告

#### 常规设置页面 (GeneralSettings)
- **外观设置**: 主题切换（亮色/暗色/跟随系统）、语言切换
- **编辑器设置**: 编辑器类型选择（VS Code/Cursor/Windsurf/Zed/Custom）、自定义命令、远程 SSH 配置、编辑器可用性检测指示器
- **Git 设置**: 分支前缀配置（含验证和预览）
- **Pull Request 设置**: 自动生成描述开关、自定义提示词
- **通知设置**: 声音通知开关、声音文件选择（暂不支持试听，Vibe-Kanban 未实现声音文件 API）
- **隐私设置**: 分析/遥测数据开关
- **任务模板**: TagManager 组件（标签 CRUD）
- **安全设置**: 重置免责声明、重置引导
- **Beta 功能**: 工作区功能、提交提醒
- **Gitea 配置**: API URL、Token、Webhook Secret（AICodex 特有）
- **审核配置**: 默认代理、自动启动、超时时间（AICodex 特有）
- **队列配置**: 最大并发、重试次数、重试延迟（AICodex 特有）

#### 项目设置页面 (ProjectSettings)
- 项目选择器（下拉菜单 + URL 同步）
- 项目常规设置（名称编辑）
- **仓库管理卡片**:
  - 仓库列表展示
  - 添加仓库按钮（打开仓库选择对话框）
  - 删除仓库按钮
  - 点击仓库跳转到仓库设置页面

#### 仓库设置页面 (RepoSettings)
- **仓库选择器**（独立下拉菜单 + URL 同步）
- 仓库常规信息:
  - **显示名称编辑** (`display_name`)
  - 仓库路径显示（只读）
- 脚本配置:
  - Dev-Server 脚本（自动扩展文本区）
  - Setup 脚本 + **并行执行开关** (`parallel_setup_script`)
  - Cleanup 脚本
  - 复制文件配置（MultiFileSearchTextarea）

#### 代理设置页面 (AgentSettings)
- **任务执行配置卡片**:
  - **执行器选择器**（选择当前使用的执行器）
  - **变体选择器**（选择执行器的变体配置）
  - **代理可用性指示器** (`AgentAvailabilityIndicator`)
- **代理配置编辑器卡片**:
  - **表单编辑器 / JSON 编辑器模式切换**
  - 执行器类型选择
  - 配置选择器（支持创建/删除配置）
  - **ExecutorConfigForm 动态表单组件**（根据执行器类型生成表单）
  - JSON 编辑器（原始配置编辑）
  - 配置文件路径显示

#### MCP 服务器配置页面 (McpSettings)
- 代理选择器
- **MCP 不支持警告提示**（当执行器不支持 MCP 时显示）
- MCP 服务器 JSON 编辑器
- 热门服务器轮播卡片（Context7、Chrome DevTools、Filesystem、Git）
- 配置保存路径提示

### 后端变更

**AICodex 后端几乎不需要变更**，因为：
1. 大部分设置 API 通过代理转发到 Vibe-Kanban
2. 代理路由 (`/api/proxy/*`) 已经存在
3. AICodex 特有的配置 API (`/api/config/*`) 已经存在

仅需要确保代理路由支持以下 Vibe-Kanban 端点：
- `GET/PUT /api/proxy/config` - 用户配置
- `GET /api/proxy/info` - 用户系统信息
- `GET/PUT /api/proxy/profiles` - 代理配置
- `GET/POST /api/proxy/mcp-config?executor={type}` - MCP 配置
- `GET /api/proxy/editors/check-availability?editor_type={type}` - 编辑器可用性
- `GET /api/proxy/agents/check-availability?executor={type}` - 代理可用性

> **注意**：声音文件 API (`/api/sounds`) 在 Vibe-Kanban 中未实现，暂不支持声音试听功能。

### 路由结构
```
/settings                → 重定向到 /settings/general
/settings/general        → 常规设置
/settings/projects       → 项目设置
/settings/repos          → 仓库设置
/settings/agents         → 代理设置
/settings/mcp            → MCP 服务器配置
```

### 不实现的功能
- **组织设置**: AICodex 为单用户系统，不需要组织和成员管理

## Impact

### 受影响的代码
- `aicodex-web/src/pages/Settings.vue` → 重构
- `aicodex-web/src/router/index.ts` → 更新路由
- `aicodex-web/src/pages/settings/` → 新增目录和页面组件
- `aicodex-web/src/components/` → 新增通用组件
  - `TagManager.vue` - 标签管理
  - `JsonEditor.vue` - JSON 编辑器封装
  - `ExecutorConfigForm.vue` - 动态执行器配置表单
  - `AgentAvailabilityIndicator.vue` - 代理可用性指示器
  - `EditorAvailabilityIndicator.vue` - 编辑器可用性指示器
  - `AutoExpandingTextarea.vue` - 自动扩展文本区
  - `MultiFileSearchTextarea.vue` - 多文件搜索文本区
- `aicodex-web/src/lib/api.ts` → 扩展代理 API，补充配置相关方法
- `aicodex-web/src/types/` → 导入 Vibe-Kanban 类型定义（`UserSystemInfo`、`Config`、`EditorConfig` 等）

### 依赖
- 前端：vue-monaco-editor（JSON 编辑器）
