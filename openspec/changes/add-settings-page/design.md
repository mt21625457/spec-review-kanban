# 技术设计文档

## Context

参照 Vibe-Kanban 的设置页面（约 2100 行代码，6 个页面），为 AICodex-Web 实现功能对等的设置系统。Vibe-Kanban 使用 React + shadcn/ui，我们需要适配到 Vue 3 + Element Plus 技术栈。

**关键架构决策**：AICodex-Web 通过代理 API 访问 Vibe-Kanban 的数据，不在 AICodex 中重复存储设置数据。

## Goals / Non-Goals

### Goals
1. 实现与 Vibe-Kanban 功能对等的设置系统（除组织设置外）
2. 提供美观的多页面布局 UI
3. 支持代理和 MCP 服务器的灵活配置
4. 实现草稿管理和撤销/保存功能
5. 支持表单编辑器和 JSON 编辑器双模式
6. 复用 Vibe-Kanban 的现有 API，避免数据重复存储

### Non-Goals
1. 不实现组织管理（AICodex 为单用户系统）
2. 不在 AICodex 中创建冗余的数据库表
3. 不实现配置版本控制

## Decisions

### 决策 1：数据架构 - 使用 Vibe-Kanban 代理 API

**原则**：设置数据存储在 Vibe-Kanban 中，AICodex 通过代理 API 访问。

| 数据类型 | 存储位置 | API 路径 |
|---------|---------|---------|
| 用户配置 | Vibe-Kanban | `/api/proxy/info`, `/api/proxy/config` |
| 项目 | Vibe-Kanban | `/api/proxy/projects/*` |
| 仓库 | Vibe-Kanban | `/api/proxy/repos/*` |
| 标签 | Vibe-Kanban | `/api/proxy/tags/*` |
| 代理配置 | Vibe-Kanban | `/api/proxy/profiles` |
| MCP 配置 | Vibe-Kanban | `/api/proxy/mcp-config` |
| Gitea 配置 | AICodex | `/api/config/gitea` |
| 审核配置 | AICodex | `/api/config/review` |
| 队列配置 | AICodex | `/api/config/queue` |

**AICodex 不需要新建数据库表**，现有的 `system_configs` 表已足够存储 AICodex 特有的配置。

### 决策 2：API 设计

#### Vibe-Kanban 代理 API（已存在，需确保完整）

```
# 用户配置
GET    /api/proxy/info                           # 获取用户系统信息（包含配置）
PUT    /api/proxy/config                         # 更新用户配置

# 编辑器/代理可用性
GET    /api/proxy/editors/check-availability?editor_type={type}
GET    /api/proxy/agents/check-availability?executor={type}

# 项目
GET    /api/proxy/projects                       # 获取项目列表
POST   /api/proxy/projects                       # 创建项目
GET    /api/proxy/projects/:id                   # 获取项目详情
PUT    /api/proxy/projects/:id                   # 更新项目
DELETE /api/proxy/projects/:id                   # 删除项目
GET    /api/proxy/projects/:id/repositories      # 获取项目关联的仓库
POST   /api/proxy/projects/:id/repositories      # 添加仓库到项目
DELETE /api/proxy/projects/:id/repositories/:repoId

# 仓库
GET    /api/proxy/repos                          # 获取仓库列表
GET    /api/proxy/repos/:id                      # 获取仓库详情
PUT    /api/proxy/repos/:id                      # 更新仓库配置

# 标签
GET    /api/proxy/tags                           # 获取标签列表
POST   /api/proxy/tags                           # 创建标签
PUT    /api/proxy/tags/:id                       # 更新标签
DELETE /api/proxy/tags/:id                       # 删除标签

# 代理配置
GET    /api/proxy/profiles                       # 获取执行器配置
PUT    /api/proxy/profiles                       # 保存执行器配置

# MCP 配置
GET    /api/proxy/mcp-config?executor={type}     # 获取 MCP 配置
POST   /api/proxy/mcp-config?executor={type}     # 保存 MCP 配置
```

> **注意**：声音文件 API (`/api/sounds`) 在 Vibe-Kanban 中未实现，暂不支持声音试听功能。

#### AICodex 特有 API（已存在）

```
# 配置管理
GET    /api/config                               # 获取所有配置
PUT    /api/config/gitea                         # 更新 Gitea 配置
POST   /api/config/gitea/test                    # 测试 Gitea 连接
PUT    /api/config/review                        # 更新审核配置
PUT    /api/config/queue                         # 更新队列配置
```

### 决策 3：前端状态管理

参照 Vibe-Kanban，实现以下状态管理模式：

```typescript
// 草稿管理模式
interface SettingsState {
  // 服务器状态
  config: UserSystemInfo | null
  loading: boolean
  error: string | null

  // 本地草稿
  draft: Config | null
  dirty: boolean
  saving: boolean
  success: boolean
}

// 计算属性
const hasUnsavedChanges = computed(() => {
  if (!draft.value || !config.value) return false
  return !isEqual(draft.value, config.value.config)
})
```

使用 Pinia store 管理全局配置状态。

### 决策 4：类型定义（对齐 Vibe-Kanban）

> **重要**：AICodex 前端需要从 Vibe-Kanban 的 `shared/types.ts` 或 `@/types` 导入以下类型。可以通过复制类型定义或建立共享类型包来实现。

```typescript
// 用户系统信息（来自 /api/proxy/info）
interface UserSystemInfo {
  config: Config
  analytics_user_id: string
  login_status: LoginStatus
  profiles: ExecutorConfigs
  environment: Environment
  capabilities: Record<string, BaseAgentCapability[]>
}

// 用户配置
interface Config {
  executor_profile: ExecutorProfileId
  editor: EditorConfig
  git_branch_prefix: string
  disclaimer_acknowledged: boolean
  onboarding_acknowledged: boolean
  analytics_enabled: boolean
  ui_language: UiLanguage
  theme_mode: ThemeMode
  sound_enabled: boolean
  sound_file: SoundFile
  terminal_shell: TerminalShell
  pr_description_prompt: string
}

// 编辑器配置
interface EditorConfig {
  editor: EditorType
  custom_command?: string
  remote_ssh_host?: string
  remote_ssh_user?: string
}

// 编辑器类型
enum EditorType {
  VS_CODE = 'vscode',
  CURSOR = 'cursor',
  WINDSURF = 'windsurf',
  ZED = 'zed',
  CUSTOM = 'custom'
}

// 主题模式
enum ThemeMode {
  LIGHT = 'light',
  DARK = 'dark',
  SYSTEM = 'system'
}

// 执行器配置
interface ExecutorConfigs {
  executors: Record<string, ExecutorConfig>
}

// 仓库配置（来自 /api/proxy/repos）
interface Repo {
  id: string
  path: string
  name: string
  display_name: string
  setup_script?: string | null
  cleanup_script?: string | null
  copy_files?: string | null
  parallel_setup_script: boolean
  dev_server_script?: string | null
  created_at: string
  updated_at: string
}

// 更新仓库请求
interface UpdateRepo {
  display_name?: string | null
  setup_script?: string | null
  cleanup_script?: string | null
  copy_files?: string | null
  parallel_setup_script?: boolean | null
  dev_server_script?: string | null
}

// 项目（来自 /api/proxy/projects）
interface Project {
  id: string
  name: string
  default_agent_working_dir?: string | null
  remote_project_id?: string | null
  created_at: string
  updated_at: string
}

// 标签（来自 /api/proxy/tags）
interface Tag {
  id: string
  tag_name: string
  content: string
  created_at: string
  updated_at: string
}

// MCP 配置
interface McpConfig {
  servers: Record<string, McpServer>
}

interface McpServer {
  type: 'stdio' | 'sse'
  command: string
  args?: string[]
  env?: Record<string, string>
}

// AICodex 特有配置
interface GiteaSettings {
  api_url?: string
  token?: string
  webhook_secret?: string
}

interface ReviewSettings {
  default_agent: string
  auto_start: boolean
  timeout_seconds: number
}

interface QueueSettings {
  max_concurrent: number
  retry_count: number
  retry_delay_seconds: number
}
```

### 决策 5：热门 MCP 服务器列表

硬编码在前端：
```typescript
const POPULAR_MCP_SERVERS = [
  {
    name: 'Context7',
    package: '@upstash/context7-mcp',
    description: '智能上下文管理',
    icon: 'context7-icon'
  },
  {
    name: 'Chrome DevTools',
    package: 'chrome-devtools-mcp@latest',
    description: '浏览器调试工具',
    icon: 'chrome-icon'
  },
  {
    name: 'Filesystem',
    package: '@anthropic/mcp-filesystem',
    description: '文件系统访问',
    icon: 'folder-icon'
  },
  {
    name: 'Git',
    package: 'mcp-git',
    description: 'Git 操作',
    icon: 'git-icon'
  },
  {
    name: 'Playwright',
    package: '@anthropic/mcp-playwright',
    description: '浏览器自动化',
    icon: 'playwright-icon'
  }
]
```

### 决策 6：组件架构

```
components/
├── settings/
│   ├── SettingsCard.vue           # 设置卡片容器
│   ├── SettingsNavItem.vue        # 导航项
│   └── StickyFooter.vue           # 粘性保存栏
├── editors/
│   ├── JsonEditor.vue             # Monaco JSON 编辑器封装
│   ├── AutoExpandingTextarea.vue  # 自动扩展文本区
│   └── MultiFileSearchTextarea.vue # 多文件搜索文本区
├── indicators/
│   ├── EditorAvailabilityIndicator.vue
│   └── AgentAvailabilityIndicator.vue
├── forms/
│   ├── ExecutorConfigForm.vue     # 动态执行器配置表单
│   └── TagManager.vue             # 标签管理组件
└── dialogs/
    ├── CreateConfigurationDialog.vue
    ├── DeleteConfigurationDialog.vue
    └── RepoPickerDialog.vue
```

## Risks / Trade-offs

### 风险 1：Monaco Editor 包体积
**缓解**: 使用动态导入，仅在需要时加载
```typescript
const JsonEditor = defineAsyncComponent(() =>
  import('@/components/editors/JsonEditor.vue')
)
```

### 风险 2：代理 API 延迟
**缓解**:
- 前端使用加载状态指示器
- 合理使用缓存
- 批量请求减少往返

### 风险 3：Vibe-Kanban 服务不可用
**缓解**:
- 前端显示友好的错误提示
- AICodex 特有配置仍可独立工作

### 风险 4：ExecutorConfigForm 动态表单复杂度
**缓解**:
- 使用 JSON Schema 定义表单结构
- 渐进式实现，先支持常用字段类型

## Open Questions

1. ~~声音文件是否需要上传功能，还是使用预设列表？~~
   - **已解决**：Vibe-Kanban 未实现声音文件 API，暂不支持声音试听功能，仅保留开关和选择器

2. 编辑器可用性检测是否需要后端支持？
   - **已解决**：使用 Vibe-Kanban 的 `/api/proxy/editors/check-availability?editor_type={type}` API

3. ExecutorConfigForm 是否需要完整实现所有字段类型？
   - **建议**：第一阶段实现常用类型（text, number, boolean, select），后续迭代

4. AICodex 如何获取 Vibe-Kanban 的类型定义？
   - **建议**：从 `aicodex-web/src/types/` 目录复制必要的类型定义，或从 Vibe-Kanban 的 `shared/types.ts` 导入
