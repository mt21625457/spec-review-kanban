/**
 * 设置页面相关类型定义
 * 对齐 Vibe-Kanban 的类型系统
 */

// 主题模式
export type ThemeMode = 'light' | 'dark' | 'system'

// UI 语言
export type UiLanguage = 'zh-CN' | 'en-US' | 'browser'

// 编辑器类型
export type EditorType = 'vscode' | 'cursor' | 'windsurf' | 'zed' | 'custom'

// 声音文件选项
export type SoundFile = 'default' | 'chime' | 'bell' | 'notification' | 'none'

// 编辑器配置
export interface EditorConfig {
  editor: EditorType
  custom_command?: string
  remote_ssh_host?: string
  remote_ssh_user?: string
}

// 用户配置（来自 Vibe-Kanban /api/proxy/info）
export interface UserConfig {
  executor_profile: string
  editor: EditorConfig
  git_branch_prefix: string
  disclaimer_acknowledged: boolean
  onboarding_acknowledged: boolean
  analytics_enabled: boolean
  ui_language: UiLanguage
  theme_mode: ThemeMode
  sound_enabled: boolean
  sound_file: SoundFile
  terminal_shell: string
  pr_description_prompt: string
  pr_auto_description: boolean
  workspace_enabled: boolean
  commit_reminder_enabled: boolean
}

// 环境信息
export interface Environment {
  os: string
  arch: string
  version: string
  data_dir: string
  config_path: string
  profiles_path: string
}

// 代理能力
export interface BaseAgentCapability {
  name: string
  description: string
  enabled: boolean
}

// 执行器配置
export interface ExecutorConfig {
  [key: string]: unknown
}

export interface ExecutorConfigs {
  executors: Record<string, ExecutorConfig>
}

// 用户系统信息（来自 /api/proxy/info）- 扩展版本包含 profiles 和 environment
export interface UserSystemInfo {
  config: UserConfig
  analytics_user_id: string
  login_status: { status: 'loggedout' } | { status: 'loggedin'; profile: { user_id: string; username: string | null; email: string } }
  profiles: ExecutorConfigs
  environment: Environment
  capabilities: Record<string, BaseAgentCapability[]>
}

// MCP 服务器配置
export interface McpServer {
  type: 'stdio' | 'sse'
  command: string
  args?: string[]
  env?: Record<string, string>
}

export interface McpConfig {
  servers: Record<string, McpServer>
}

// 热门 MCP 服务器
export interface PopularMcpServer {
  name: string
  package: string
  description: string
  icon: string
  config: McpServer
}

// 预配置的热门 MCP 服务器列表
export const POPULAR_MCP_SERVERS: PopularMcpServer[] = [
  {
    name: 'Context7',
    package: '@upstash/context7-mcp',
    description: '智能上下文管理',
    icon: 'brain',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['-y', '@upstash/context7-mcp'],
    },
  },
  {
    name: 'Chrome DevTools',
    package: 'chrome-devtools-mcp',
    description: '浏览器调试工具',
    icon: 'chrome',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['-y', 'chrome-devtools-mcp@latest'],
    },
  },
  {
    name: 'Filesystem',
    package: '@anthropic/mcp-filesystem',
    description: '文件系统访问',
    icon: 'folder',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['-y', '@anthropic/mcp-filesystem'],
    },
  },
  {
    name: 'Git',
    package: 'mcp-git',
    description: 'Git 操作',
    icon: 'git-branch',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['-y', 'mcp-git'],
    },
  },
  {
    name: 'Playwright',
    package: '@anthropic/mcp-playwright',
    description: '浏览器自动化',
    icon: 'play',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['-y', '@anthropic/mcp-playwright'],
    },
  },
]

// 编辑器类型选项
export const EDITOR_TYPES: { value: EditorType; label: string }[] = [
  { value: 'vscode', label: 'VS Code' },
  { value: 'cursor', label: 'Cursor' },
  { value: 'windsurf', label: 'Windsurf' },
  { value: 'zed', label: 'Zed' },
  { value: 'custom', label: 'Custom' },
]

// 支持远程 SSH 的编辑器
export const SSH_SUPPORTED_EDITORS: EditorType[] = ['vscode', 'cursor']

// 主题选项
export const THEME_OPTIONS: { value: ThemeMode; labelKey: string }[] = [
  { value: 'light', labelKey: 'settings.general.appearance.themeLight' },
  { value: 'dark', labelKey: 'settings.general.appearance.themeDark' },
  { value: 'system', labelKey: 'settings.general.appearance.themeSystem' },
]

// 语言选项
export const LANGUAGE_OPTIONS: { value: UiLanguage; label: string }[] = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en-US', label: 'English' },
  { value: 'browser', label: '跟随浏览器' },
]
