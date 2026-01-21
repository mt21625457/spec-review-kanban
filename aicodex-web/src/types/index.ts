/**
 * Core types for Agent Orchestration
 * Adapted from vibe-kanban shared/types.ts
 */

// Task Status
export type TaskStatus = 'todo' | 'inprogress' | 'inreview' | 'done' | 'cancelled'

// Project
export interface Project {
  id: string
  name: string
  default_agent_working_dir: string | null
  remote_project_id: string | null
  created_at: string
  updated_at: string
}

export interface CreateProject {
  name: string
  repositories: CreateProjectRepo[]
}

export interface UpdateProject {
  name: string | null
}

// Repository
export interface Repo {
  id: string
  path: string
  name: string
  display_name: string
  setup_script: string | null
  cleanup_script: string | null
  copy_files: string | null
  parallel_setup_script: boolean
  dev_server_script: string | null
  created_at: string
  updated_at: string
}

export interface CreateProjectRepo {
  display_name: string
  git_repo_path: string
}

export interface RepoWithTargetBranch extends Repo {
  target_branch: string
}

export interface UpdateRepo {
  display_name?: string | null
  setup_script?: string | null
  cleanup_script?: string | null
  copy_files?: string | null
  parallel_setup_script?: boolean | null
  dev_server_script?: string | null
}

// Task
export interface Task {
  id: string
  project_id: string
  title: string
  description: string | null
  status: TaskStatus
  parent_workspace_id: string | null
  shared_task_id: string | null
  created_at: string
  updated_at: string
}

export interface TaskWithAttemptStatus extends Task {
  has_in_progress_attempt: boolean
  last_attempt_failed: boolean
  executor: string
}

export interface CreateTask {
  project_id: string
  title: string
  description: string | null
  status: TaskStatus | null
  parent_workspace_id?: string | null
  image_ids?: string[] | null
  shared_task_id?: string | null
}

export interface UpdateTask {
  title?: string | null
  description?: string | null
  status?: TaskStatus | null
  parent_workspace_id?: string | null
  image_ids?: string[] | null
}

// Tag
export interface Tag {
  id: string
  tag_name: string
  content: string
  created_at: string
  updated_at: string
}

export interface CreateTag {
  tag_name: string
  content: string
}

export interface UpdateTag {
  tag_name?: string | null
  content?: string | null
}

// Workspace (Task Attempt)
export interface Workspace {
  id: string
  task_id: string
  container_ref: string | null
  branch: string
  agent_working_dir: string | null
  setup_completed_at: string | null
  created_at: string
  updated_at: string
  archived: boolean
  pinned: boolean
  name: string | null
}

export interface WorkspaceWithStatus extends Workspace {
  is_running: boolean
  is_errored: boolean
}

export interface CreateTaskAttemptBody {
  task_id: string
  executor: string
  repos?: { repo_id: string; target_branch: string }[]
}

// Session
export interface Session {
  id: string
  workspace_id: string
  executor: string | null
  created_at: string
  updated_at: string
}

// Execution Process
export type ExecutionProcessStatus = 'running' | 'completed' | 'failed' | 'killed'
export type ExecutionProcessRunReason = 'setupscript' | 'cleanupscript' | 'codingagent' | 'devserver'

export interface ExecutionProcess {
  id: string
  session_id: string
  run_reason: ExecutionProcessRunReason
  executor_action: string
  status: ExecutionProcessStatus
  exit_code: number | null
  prompt: string | null
  variant: string | null
  total_cost_usd: string | null
  input_tokens: number | null
  output_tokens: number | null
  cache_read_tokens: number | null
  cache_write_tokens: number | null
  dev_server_port: number | null
  started_at: string
  finished_at: string | null
  created_at: string
  updated_at: string
}

export interface ExecutionProcessRepoState {
  id: string
  execution_process_id: string
  repo_id: string
  before_head_commit: string | null
  after_head_commit: string | null
  merge_commit: string | null
  created_at: string
  updated_at: string
}

// Shared Task
export interface SharedTask {
  id: string
  organization_id: string
  project_id: string
  creator_user_id: string | null
  assignee_user_id: string | null
  deleted_by_user_id: string | null
  title: string
  description: string | null
  status: TaskStatus
  deleted_at: string | null
  shared_at: string | null
  created_at: string
  updated_at: string
}

export interface SharedTaskResponse {
  task: SharedTask
  user: UserData | null
}

export interface UserData {
  user_id: string
  first_name: string | null
  last_name: string | null
  username: string | null
}

// Diff
export type DiffChangeKind = 'added' | 'deleted' | 'modified' | 'renamed' | 'copied' | 'permissionChange'

export interface Diff {
  change: DiffChangeKind
  oldPath: string | null
  newPath: string | null
  oldContent: string | null
  newContent: string | null
}

// Git
export interface GitBranch {
  name: string
  is_head: boolean
  is_remote: boolean
  upstream: string | null
}

export interface RepoBranchStatus {
  repo_id: string
  behind_count: number
  ahead_count: number
  has_conflicts: boolean
}

// PR
export type MergeStatus = 'open' | 'merged' | 'closed' | 'unknown'

export interface PullRequestInfo {
  number: number
  url: string
  status: MergeStatus
  merged_at: string | null
  merge_commit_sha: string | null
}

// API Response
export interface ApiResponse<T, E = unknown> {
  success: boolean
  data: T | null
  error_data: E | null
  message: string | null
}

// Config
export interface Config {
  default_editor: string | null
  default_executor: string | null
}

export interface UserSystemInfo {
  config: Config
  analytics_user_id: string
  login_status: LoginStatus
}

export type LoginStatus =
  | { status: 'loggedout' }
  | { status: 'loggedin'; profile: ProfileResponse }

export interface ProfileResponse {
  user_id: string
  username: string | null
  email: string
}

// Search
export type SearchMode = 'files' | 'content'

export interface SearchResult {
  path: string
  is_file: boolean
  match_type: 'FileName' | 'DirectoryName' | 'FullPath'
}

// WebSocket Messages
export interface TasksSnapshot {
  Snapshot: {
    tasks: Record<string, Task>
    timestamp: string
  }
}

export interface TasksJsonPatch {
  JsonPatch: Array<{
    op: 'add' | 'remove' | 'replace' | 'move' | 'copy' | 'test'
    path: string
    value?: unknown
    from?: string
  }>
}

export type TasksStreamMessage = TasksSnapshot | TasksJsonPatch

// Executor
export type BaseCodingAgent = 'claude-code' | 'codex' | 'custom'

export interface AvailabilityInfo {
  available: boolean
  message?: string
}

// ============================================
// Authentication Types (aicodex multi-instance)
// ============================================

export type UserRole = 'admin' | 'user'

export interface UserInfo {
  id: string
  username: string
  email: string | null
  display_name: string | null
  role: UserRole
  is_active: boolean
  current_instance_id: string | null
  created_at: string
  last_login_at: string | null
}

export interface InstanceInfo {
  id: string
  name: string
  description: string | null
  port: number
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'error'
  health_status: 'unknown' | 'healthy' | 'unhealthy'
  auto_start: boolean
  max_users: number | null
  user_count?: number
  created_at: string
  last_health_check: string | null
  /** 最后一次错误信息 */
  last_error: string | null
  /** 最后一次错误时间 */
  last_error_at: string | null
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  user: UserInfo
  instances: InstanceInfo[]
  current_instance_id: string | null
}

export interface RegisterRequest {
  username: string
  email?: string
  password: string
  display_name?: string
}

export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}

export interface SwitchInstanceRequest {
  instance_id: string
}
