const API_BASE = '/api'
const PROXY_BASE = '/api/proxy'

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  message?: string
}

// ============================================
// Existing aicodex types
// ============================================

// 配置类型
export interface GiteaSettings {
  api_url?: string
  token?: string
  webhook_secret?: string
}

export interface ReviewSettings {
  default_agent: string
  auto_start: boolean
  timeout_seconds: number
}

export interface QueueSettings {
  max_concurrent: number
  retry_count: number
  retry_delay_seconds: number
}

export interface AllSettings {
  gitea: GiteaSettings
  review: ReviewSettings
  queue: QueueSettings
}

// 仓库映射类型
export interface RepoMapping {
  id: string
  gitea_repo: string
  local_path: string
  vibe_project_id?: string
  agent_type: string
  executor_profile_id?: string
  custom_prompt?: string
  is_enabled: boolean
  created_at: string
  updated_at: string
}

export interface CreateRepoMapping {
  gitea_repo: string
  local_path: string
  vibe_project_id?: string
  agent_type?: string
  executor_profile_id?: string
  custom_prompt?: string
  is_enabled?: boolean
}

// 审核类型
export interface ReviewRun {
  id: string
  repo_mapping_id: string
  gitea_repo?: string
  gitea_pr_number: number
  gitea_pr_url?: string
  vibe_task_id?: string
  vibe_workspace_id?: string
  agent_type?: string
  status: string
  started_at?: string
  completed_at?: string
  error_message?: string
  created_at: string
  updated_at: string
}

export interface ReviewEvent {
  id: string
  review_run_id: string
  event_type: string
  event_data?: string
  created_at: string
}

// Webhook 审计
export interface WebhookAudit {
  id: string
  gitea_repo: string
  event_type: string
  payload_hash: string
  status: string
  error_message?: string
  created_at: string
}

// 连接测试结果
export interface ConnectionTestResult {
  success: boolean
  user?: {
    id: number
    login: string
    full_name?: string
    email?: string
  }
  error?: string
}

async function request<T>(url: string, options?: RequestInit): Promise<ApiResponse<T>> {
  try {
    const response = await fetch(API_BASE + url, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    })

    const data = await response.json()
    return data
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : '请求失败',
    }
  }
}

// 配置 API
export const configApi = {
  getAll: () => request<AllSettings>('/config'),
  updateGitea: (settings: GiteaSettings) =>
    request<GiteaSettings>('/config/gitea', {
      method: 'PUT',
      body: JSON.stringify(settings),
    }),
  testGiteaConnection: () =>
    request<ConnectionTestResult>('/config/gitea/test', { method: 'POST' }),
  updateReview: (settings: ReviewSettings) =>
    request<ReviewSettings>('/config/review', {
      method: 'PUT',
      body: JSON.stringify(settings),
    }),
  updateQueue: (settings: QueueSettings) =>
    request<QueueSettings>('/config/queue', {
      method: 'PUT',
      body: JSON.stringify(settings),
    }),
}

// 仓库映射 API
export const repoMappingApi = {
  list: () => request<RepoMapping[]>('/repo-mappings'),
  get: (id: string) => request<RepoMapping>(`/repo-mappings/${id}`),
  create: (data: CreateRepoMapping) =>
    request<RepoMapping>('/repo-mappings', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  update: (id: string, data: Partial<RepoMapping>) =>
    request<RepoMapping>(`/repo-mappings/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  delete: (id: string) =>
    request<void>(`/repo-mappings/${id}`, { method: 'DELETE' }),
  sync: (id: string) =>
    request<RepoMapping>(`/repo-mappings/${id}/sync`, { method: 'POST' }),
}

// 审核 API
export const reviewApi = {
  list: (limit = 50) => request<ReviewRun[]>(`/reviews?limit=${limit}`),
  get: (id: string) =>
    request<{ review: ReviewRun; events: ReviewEvent[] }>(`/reviews/${id}`),
  rerun: (id: string) =>
    request<ReviewRun>(`/reviews/${id}/rerun`, { method: 'POST' }),
  cancel: (id: string) =>
    request<ReviewRun>(`/reviews/${id}/cancel`, { method: 'POST' }),
  getEvents: (id: string) => request<ReviewEvent[]>(`/reviews/${id}/events`),
}

// Webhook API
export const webhookApi = {
  listAudits: (limit = 50, status?: string) => {
    const params = new URLSearchParams({ limit: String(limit) })
    if (status) params.append('status', status)
    return request<WebhookAudit[]>(`/webhooks/audits?${params}`)
  },
}

// ============================================
// Vibe-Kanban Proxy APIs (通过 aicodex 代理访问)
// ============================================

import type {
  Project,
  CreateProject,
  UpdateProject,
  Task,
  CreateTask,
  UpdateTask,
  TaskWithAttemptStatus,
  Tag,
  CreateTag,
  UpdateTag,
  Workspace,
  CreateTaskAttemptBody,
  Session,
  ExecutionProcess,
  ExecutionProcessRepoState,
  Repo,
  UpdateRepo,
  CreateProjectRepo,
  RepoWithTargetBranch,
  GitBranch,
  RepoBranchStatus,
  SearchResult,
  SearchMode,
  UserSystemInfo,
  AvailabilityInfo,
  BaseCodingAgent,
  SharedTaskResponse,
  // Authentication types
  UserInfo,
  InstanceInfo,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  ChangePasswordRequest,
  SwitchInstanceRequest,
} from '@/types'

// API Error class
export class ApiError<E = unknown> extends Error {
  public status?: number
  public error_data?: E

  constructor(
    message: string,
    public statusCode?: number,
    public response?: Response,
    error_data?: E
  ) {
    super(message)
    this.name = 'ApiError'
    this.status = statusCode
    this.error_data = error_data
  }
}

// Proxy request helper
async function proxyRequest<T>(
  url: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(PROXY_BASE + url, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  })

  if (!response.ok) {
    let errorMessage = `Request failed with status ${response.status}`
    try {
      const errorData = await response.json()
      if (errorData.message) {
        errorMessage = errorData.message
      }
    } catch {
      errorMessage = response.statusText || errorMessage
    }
    throw new ApiError(errorMessage, response.status, response)
  }

  if (response.status === 204) {
    return undefined as T
  }

  const result = await response.json()

  if (result.success === false) {
    throw new ApiError(
      result.message || 'API request failed',
      response.status,
      response,
      result.error_data
    )
  }

  return result.data ?? result
}

// Projects API (vibe-kanban)
export const projectsApi = {
  list: () => proxyRequest<Project[]>('/projects'),

  get: (id: string) => proxyRequest<Project>(`/projects/${id}`),

  create: (data: CreateProject) =>
    proxyRequest<Project>('/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateProject) =>
    proxyRequest<Project>(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    proxyRequest<void>(`/projects/${id}`, { method: 'DELETE' }),

  getRepositories: (projectId: string) =>
    proxyRequest<Repo[]>(`/projects/${projectId}/repositories`),

  addRepository: (projectId: string, data: CreateProjectRepo) =>
    proxyRequest<Repo>(`/projects/${projectId}/repositories`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  deleteRepository: (projectId: string, repoId: string) =>
    proxyRequest<void>(`/projects/${projectId}/repositories/${repoId}`, {
      method: 'DELETE',
    }),

  searchFiles: (id: string, query: string, mode?: SearchMode) => {
    const modeParam = mode ? `&mode=${encodeURIComponent(mode)}` : ''
    return proxyRequest<SearchResult[]>(
      `/projects/${id}/search?q=${encodeURIComponent(query)}${modeParam}`
    )
  },
}

// Tasks API (vibe-kanban)
export const tasksApi = {
  getByProject: (projectId: string) =>
    proxyRequest<TaskWithAttemptStatus[]>(`/tasks?project_id=${projectId}`),

  getById: (taskId: string) => proxyRequest<Task>(`/tasks/${taskId}`),

  create: (data: CreateTask) =>
    proxyRequest<Task>('/tasks', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  createAndStart: (data: CreateTask & { executor: string }) =>
    proxyRequest<TaskWithAttemptStatus>('/tasks/create-and-start', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (taskId: string, data: UpdateTask) =>
    proxyRequest<Task>(`/tasks/${taskId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (taskId: string) =>
    proxyRequest<void>(`/tasks/${taskId}`, { method: 'DELETE' }),

  share: (taskId: string) =>
    proxyRequest<{ shared_task_id: string }>(`/tasks/${taskId}/share`, {
      method: 'POST',
    }),

  unshare: (sharedTaskId: string) =>
    proxyRequest<void>(`/shared-tasks/${sharedTaskId}`, { method: 'DELETE' }),

  getSharedTask: (sharedTaskId: string) =>
    proxyRequest<SharedTaskResponse>(`/shared-tasks/${sharedTaskId}`),

  linkSharedTask: (data: { shared_task_id: string; project_id: string }) =>
    proxyRequest<Task>('/shared-tasks/link-to-local', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
}

// Tags API (vibe-kanban)
export const tagsApi = {
  list: (search?: string) => {
    const queryParam = search ? `?search=${encodeURIComponent(search)}` : ''
    return proxyRequest<Tag[]>(`/tags${queryParam}`)
  },

  create: (data: CreateTag) =>
    proxyRequest<Tag>('/tags', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (tagId: string, data: UpdateTag) =>
    proxyRequest<Tag>(`/tags/${tagId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (tagId: string) =>
    proxyRequest<void>(`/tags/${tagId}`, { method: 'DELETE' }),
}

// Workspaces (Task Attempts) API (vibe-kanban)
export const workspacesApi = {
  getAll: (taskId?: string) => {
    const queryParam = taskId ? `?task_id=${taskId}` : ''
    return proxyRequest<Workspace[]>(`/task-attempts${queryParam}`)
  },

  get: (attemptId: string) =>
    proxyRequest<Workspace>(`/task-attempts/${attemptId}`),

  create: (data: CreateTaskAttemptBody) =>
    proxyRequest<Workspace>('/task-attempts', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (
    attemptId: string,
    data: { archived?: boolean; pinned?: boolean; name?: string }
  ) =>
    proxyRequest<Workspace>(`/task-attempts/${attemptId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (attemptId: string) =>
    proxyRequest<void>(`/task-attempts/${attemptId}`, { method: 'DELETE' }),

  stop: (attemptId: string) =>
    proxyRequest<void>(`/task-attempts/${attemptId}/stop`, { method: 'POST' }),

  getRepos: (attemptId: string) =>
    proxyRequest<RepoWithTargetBranch[]>(`/task-attempts/${attemptId}/repos`),

  getBranchStatus: (attemptId: string) =>
    proxyRequest<RepoBranchStatus[]>(`/task-attempts/${attemptId}/branch-status`),

  push: (attemptId: string, data: { repo_id: string; force?: boolean }) =>
    proxyRequest<void>(`/task-attempts/${attemptId}/push`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  createPR: (
    attemptId: string,
    data: { repo_id: string; title: string; body: string }
  ) =>
    proxyRequest<string>(`/task-attempts/${attemptId}/pr`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  startDevServer: (attemptId: string) =>
    proxyRequest<void>(`/task-attempts/${attemptId}/start-dev-server`, {
      method: 'POST',
    }),

  searchFiles: (workspaceId: string, query: string, mode?: string) => {
    const modeParam = mode ? `&mode=${encodeURIComponent(mode)}` : ''
    return proxyRequest<SearchResult[]>(
      `/task-attempts/${workspaceId}/search?q=${encodeURIComponent(query)}${modeParam}`
    )
  },
}

// Sessions API (vibe-kanban)
export const sessionsApi = {
  getByWorkspace: (workspaceId: string) =>
    proxyRequest<Session[]>(`/sessions?workspace_id=${workspaceId}`),

  get: (sessionId: string) => proxyRequest<Session>(`/sessions/${sessionId}`),

  create: (data: { workspace_id: string; executor?: string }) =>
    proxyRequest<Session>('/sessions', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  followUp: (
    sessionId: string,
    data: { prompt: string; variant?: string | null }
  ) =>
    proxyRequest<ExecutionProcess>(`/sessions/${sessionId}/follow-up`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
}

// Execution Processes API (vibe-kanban)
export const executionProcessesApi = {
  get: (processId: string) =>
    proxyRequest<ExecutionProcess>(`/execution-processes/${processId}`),

  getRepoStates: (processId: string) =>
    proxyRequest<ExecutionProcessRepoState[]>(
      `/execution-processes/${processId}/repo-states`
    ),

  stop: (processId: string) =>
    proxyRequest<void>(`/execution-processes/${processId}/stop`, {
      method: 'POST',
    }),
}

// Repos API (vibe-kanban)
export const reposApi = {
  list: () => proxyRequest<Repo[]>('/repos'),

  get: (repoId: string) => proxyRequest<Repo>(`/repos/${repoId}`),

  update: (repoId: string, data: UpdateRepo) =>
    proxyRequest<Repo>(`/repos/${repoId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  register: (data: { path: string; display_name?: string }) =>
    proxyRequest<Repo>('/repos', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  init: (data: { parent_path: string; folder_name: string }) =>
    proxyRequest<Repo>('/repos/init', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getBranches: (repoId: string) =>
    proxyRequest<GitBranch[]>(`/repos/${repoId}/branches`),
}

// Directory Entry type for filesystem API
export interface DirectoryEntry {
  name: string
  path: string
  is_dir: boolean
  is_git_repo?: boolean
}

export interface DirectoryListResponse {
  path: string
  entries: DirectoryEntry[]
  parent_path: string | null
}

// FileSystem API (vibe-kanban)
export const fileSystemApi = {
  listDirectory: (path?: string) => {
    const queryParam = path ? `?path=${encodeURIComponent(path)}` : ''
    return proxyRequest<DirectoryListResponse>(`/filesystem/directory${queryParam}`)
  },

  listGitRepos: (path?: string) => {
    const queryParam = path ? `?path=${encodeURIComponent(path)}` : ''
    return proxyRequest<DirectoryEntry[]>(`/filesystem/git-repos${queryParam}`)
  },
}

// System Info API (vibe-kanban)
export const systemApi = {
  getInfo: () => proxyRequest<UserSystemInfo>('/info'),

  checkAgentAvailability: (agent: BaseCodingAgent) =>
    proxyRequest<AvailabilityInfo>(
      `/agents/check-availability?executor=${encodeURIComponent(agent)}`
    ),

  updateConfig: (config: Partial<{
    executor_profile: string
    theme_mode: string
    ui_language: string
    sound_enabled: boolean
    sound_file: string
    terminal_shell: string
    git_branch_prefix: string
    pr_description_prompt: string
    pr_auto_description: boolean
    analytics_enabled: boolean
    workspace_enabled: boolean
    commit_reminder_enabled: boolean
    disclaimer_acknowledged: boolean
    onboarding_acknowledged: boolean
    editor: {
      editor: string
      custom_command?: string
      remote_ssh_host?: string
      remote_ssh_user?: string
    }
  }>) =>
    proxyRequest<void>('/config', {
      method: 'PUT',
      body: JSON.stringify(config),
    }),
}

// MCP Config API (vibe-kanban)
export interface McpServer {
  type: 'stdio' | 'sse'
  command: string
  args?: string[]
  env?: Record<string, string>
}

export interface McpConfig {
  servers: Record<string, McpServer>
}

export const mcpApi = {
  getConfig: (executor: string) =>
    proxyRequest<McpConfig>(`/mcp-config?executor=${encodeURIComponent(executor)}`),

  updateConfig: (executor: string, config: McpConfig) =>
    proxyRequest<void>(`/mcp-config?executor=${encodeURIComponent(executor)}`, {
      method: 'POST',
      body: JSON.stringify(config),
    }),
}

// Editor Availability API (vibe-kanban)
export const editorApi = {
  checkAvailability: (editorType: string) =>
    proxyRequest<AvailabilityInfo>(
      `/editors/check-availability?editor_type=${encodeURIComponent(editorType)}`
    ),
}

// Profiles API (vibe-kanban) - 执行器配置管理
export interface ExecutorProfile {
  [key: string]: unknown
}

export interface ProfilesConfig {
  executors: Record<string, ExecutorProfile>
}

export const profilesApi = {
  get: () => proxyRequest<ProfilesConfig>('/profiles'),

  update: (profiles: ProfilesConfig) =>
    proxyRequest<void>('/profiles', {
      method: 'PUT',
      body: JSON.stringify(profiles),
    }),
}

// WebSocket URLs
export const wsUrls = {
  tasksStream: (projectId: string) =>
    `/api/proxy/tasks/stream/ws?project_id=${projectId}`,

  attemptLogs: (attemptId: string) =>
    `/api/proxy/task-attempts/${attemptId}/logs/ws`,

  executionProcess: (processId: string) =>
    `/api/proxy/execution-processes/${processId}/stream/ws`,
}

// ============================================
// Authentication APIs (aicodex multi-instance)
// ============================================

// Token 管理
let authToken: string | null = null

export function setAuthToken(token: string | null) {
  authToken = token
  if (token) {
    localStorage.setItem('auth_token', token)
    // 也设置 cookie 供代理使用
    document.cookie = `auth_token=${token}; path=/; max-age=${60 * 60 * 24 * 7}` // 7 天
  } else {
    localStorage.removeItem('auth_token')
    document.cookie = 'auth_token=; path=/; max-age=0'
  }
}

export function getAuthToken(): string | null {
  if (authToken) return authToken
  return localStorage.getItem('auth_token')
}

// 带认证的请求
async function authRequest<T>(url: string, options?: RequestInit): Promise<ApiResponse<T>> {
  const token = getAuthToken()
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...options?.headers as Record<string, string>,
  }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  try {
    const response = await fetch(API_BASE + url, {
      ...options,
      headers,
    })

    const data = await response.json()
    return data
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : '请求失败',
    }
  }
}

// 认证 API
export const authApi = {
  // 用户注册
  register: (data: RegisterRequest) =>
    authRequest<UserInfo>('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // 用户登录
  login: async (data: LoginRequest): Promise<ApiResponse<LoginResponse>> => {
    const response = await authRequest<LoginResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    })
    if (response.success && response.data?.token) {
      setAuthToken(response.data.token)
    }
    return response
  },

  // 用户登出
  logout: async (): Promise<ApiResponse<void>> => {
    const response = await authRequest<void>('/auth/logout', { method: 'POST' })
    setAuthToken(null)
    return response
  },

  // 获取当前用户
  me: () =>
    authRequest<{ user: UserInfo; instances: InstanceInfo[] }>('/auth/me'),

  // 修改密码
  changePassword: (data: ChangePasswordRequest) =>
    authRequest<void>('/auth/password', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
}

// 用户实例切换 API
export const myInstancesApi = {
  // 获取当前用户的所有实例
  list: () =>
    authRequest<{ instances: InstanceInfo[]; current_instance_id: string | null }>(
      '/my-instances'
    ),

  // 获取当前实例
  getCurrent: () =>
    authRequest<{ instance: InstanceInfo }>('/my-instances/current'),

  // 切换当前实例
  switchInstance: (data: SwitchInstanceRequest) =>
    authRequest<{ instance: InstanceInfo }>('/my-instances/current', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // 当前实例健康检查
  currentHealth: () =>
    authRequest<{ instance_id: string; health_status: string }>(
      '/my-instances/current/health'
    ),
}

// ============================================
// Admin APIs (管理员功能)
// ============================================

// 用户管理请求类型
export interface CreateUserRequest {
  username: string
  email?: string
  password: string
  display_name?: string
  role?: 'admin' | 'user'
}

export interface UpdateUserRequest {
  email?: string
  display_name?: string
  role?: 'admin' | 'user'
}

export interface AssignInstancesRequest {
  instance_ids: string[]
}

// 实例管理请求类型
export interface CreateInstanceRequest {
  name: string
  description?: string
  auto_start?: boolean
  max_users?: number
}

export interface UpdateInstanceRequest {
  name?: string
  description?: string
  auto_start?: boolean
  max_users?: number
}

// AI 智能体配置类型
export interface AgentConfig {
  id: string
  instance_id: string
  agent_type: string
  is_enabled: boolean
  api_key_masked?: string
  config_json?: string
  rate_limit_rpm?: number
  created_at: string
  updated_at: string
}

export interface SetAgentConfigRequest {
  is_enabled?: boolean
  api_key?: string
  config_json?: string
  rate_limit_rpm?: number
}

// 用户管理 API（管理员）
export const usersApi = {
  // 列出所有用户
  list: () =>
    authRequest<{ users: UserInfo[] }>('/users'),

  // 获取用户详情
  get: (userId: string) =>
    authRequest<{ user: UserInfo; instances: InstanceInfo[] }>(`/users/${userId}`),

  // 创建用户
  create: (data: CreateUserRequest) =>
    authRequest<{ user: UserInfo }>('/users', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // 更新用户
  update: (userId: string, data: UpdateUserRequest) =>
    authRequest<{ user: UserInfo }>(`/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // 删除用户
  delete: (userId: string) =>
    authRequest<void>(`/users/${userId}`, { method: 'DELETE' }),

  // 激活/停用用户
  setActive: (userId: string, isActive: boolean) =>
    authRequest<void>(`/users/${userId}/activate`, {
      method: 'PUT',
      body: JSON.stringify({ is_active: isActive }),
    }),

  // 重置密码
  resetPassword: (userId: string, newPassword: string) =>
    authRequest<void>(`/users/${userId}/password`, {
      method: 'PUT',
      body: JSON.stringify({ new_password: newPassword }),
    }),

  // 获取用户的实例列表
  getInstances: (userId: string) =>
    authRequest<{ instances: InstanceInfo[] }>(`/users/${userId}/instances`),

  // 分配实例给用户
  assignInstances: (userId: string, instanceIds: string[]) =>
    authRequest<void>(`/users/${userId}/instances`, {
      method: 'POST',
      body: JSON.stringify({ instance_ids: instanceIds }),
    }),

  // 取消实例分配
  unassignInstance: (userId: string, instanceId: string) =>
    authRequest<void>(`/users/${userId}/instances/${instanceId}`, {
      method: 'DELETE',
    }),
}

// 实例管理 API（管理员）
export const instancesApi = {
  // 列出所有实例
  list: () =>
    authRequest<{ instances: InstanceInfo[] }>('/instances'),

  // 获取实例详情
  get: (instanceId: string) =>
    authRequest<{ instance: InstanceInfo; users: UserInfo[]; agents: AgentConfig[] }>(
      `/instances/${instanceId}`
    ),

  // 创建实例
  create: (data: CreateInstanceRequest) =>
    authRequest<{ instance: InstanceInfo }>('/instances', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  // 更新实例
  update: (instanceId: string, data: UpdateInstanceRequest) =>
    authRequest<{ instance: InstanceInfo }>(`/instances/${instanceId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // 删除实例
  delete: (instanceId: string) =>
    authRequest<void>(`/instances/${instanceId}`, { method: 'DELETE' }),

  // 启动实例
  start: (instanceId: string) =>
    authRequest<{ instance: InstanceInfo }>(`/instances/${instanceId}/start`, {
      method: 'POST',
    }),

  // 停止实例
  stop: (instanceId: string) =>
    authRequest<{ instance: InstanceInfo }>(`/instances/${instanceId}/stop`, {
      method: 'POST',
    }),

  // 重启实例
  restart: (instanceId: string) =>
    authRequest<{ instance: InstanceInfo }>(`/instances/${instanceId}/restart`, {
      method: 'POST',
    }),

  // 健康检查
  health: (instanceId: string) =>
    authRequest<{ health_status: string }>(`/instances/${instanceId}/health`),

  // 获取实例用户
  getUsers: (instanceId: string) =>
    authRequest<{ users: UserInfo[] }>(`/instances/${instanceId}/users`),

  // 获取智能体配置列表
  getAgents: (instanceId: string) =>
    authRequest<{ agents: AgentConfig[] }>(`/instances/${instanceId}/agents`),

  // 设置智能体配置
  setAgentConfig: (instanceId: string, agentType: string, data: SetAgentConfigRequest) =>
    authRequest<{ agent: AgentConfig }>(`/instances/${instanceId}/agents/${agentType}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  // 测试智能体连接
  testAgentConnection: (instanceId: string, agentType: string) =>
    authRequest<{ connection_ok: boolean }>(`/instances/${instanceId}/agents/${agentType}/test`, {
      method: 'POST',
    }),
}

