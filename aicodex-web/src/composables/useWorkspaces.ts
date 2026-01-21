/**
 * useWorkspaces - Vue Query hooks for workspace (task attempt) management
 */
import { computed, type Ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { workspacesApi, sessionsApi } from '@/lib/api'
import { queryKeys } from '@/lib/queryClient'
import type {
  Workspace,
  CreateTaskAttemptBody,
} from '@/types'

/**
 * Fetch all workspaces for a task
 */
export function useWorkspaces(taskId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      taskId.value ? queryKeys.workspaces.byTask(taskId.value) : ['workspaces', null]
    ),
    queryFn: () => {
      if (!taskId.value) throw new Error('Task ID is required')
      return workspacesApi.getAll(taskId.value)
    },
    enabled: computed(() => !!taskId.value),
  })
}

/**
 * Fetch a single workspace by ID
 */
export function useWorkspace(workspaceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      workspaceId.value ? queryKeys.workspaces.detail(workspaceId.value) : ['workspaces', null]
    ),
    queryFn: () => {
      if (!workspaceId.value) throw new Error('Workspace ID is required')
      return workspacesApi.get(workspaceId.value)
    },
    enabled: computed(() => !!workspaceId.value),
  })
}

/**
 * Fetch workspace repositories
 */
export function useWorkspaceRepos(workspaceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      workspaceId.value ? queryKeys.workspaces.repos(workspaceId.value) : ['workspaces', null, 'repos']
    ),
    queryFn: () => {
      if (!workspaceId.value) throw new Error('Workspace ID is required')
      return workspacesApi.getRepos(workspaceId.value)
    },
    enabled: computed(() => !!workspaceId.value),
  })
}

/**
 * Fetch workspace branch status
 */
export function useWorkspaceBranchStatus(workspaceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      workspaceId.value ? queryKeys.workspaces.branchStatus(workspaceId.value) : ['workspaces', null, 'branchStatus']
    ),
    queryFn: () => {
      if (!workspaceId.value) throw new Error('Workspace ID is required')
      return workspacesApi.getBranchStatus(workspaceId.value)
    },
    enabled: computed(() => !!workspaceId.value),
    refetchInterval: 10000, // Refetch every 10 seconds
  })
}

/**
 * Create a new workspace (task attempt)
 */
export function useCreateWorkspace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateTaskAttemptBody) => workspacesApi.create(data),
    onSuccess: (newWorkspace) => {
      // Update cache
      queryClient.setQueryData(queryKeys.workspaces.detail(newWorkspace.id), newWorkspace)
      // Invalidate task workspaces list
      queryClient.invalidateQueries({
        queryKey: queryKeys.workspaces.byTask(newWorkspace.task_id),
      })
    },
  })
}

/**
 * Update a workspace
 */
export function useUpdateWorkspace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      workspaceId,
      data,
    }: {
      workspaceId: string
      data: { archived?: boolean; pinned?: boolean; name?: string }
    }) => workspacesApi.update(workspaceId, data),
    onSuccess: (updatedWorkspace, { workspaceId }) => {
      queryClient.setQueryData(queryKeys.workspaces.detail(workspaceId), updatedWorkspace)
    },
  })
}

/**
 * Delete a workspace
 */
export function useDeleteWorkspace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (workspaceId: string) => workspacesApi.delete(workspaceId),
    onSuccess: (_, workspaceId) => {
      queryClient.removeQueries({ queryKey: queryKeys.workspaces.detail(workspaceId) })
    },
  })
}

/**
 * Stop a running workspace
 */
export function useStopWorkspace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (workspaceId: string) => workspacesApi.stop(workspaceId),
    onSuccess: (_, workspaceId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.workspaces.detail(workspaceId) })
    },
  })
}

/**
 * Push workspace changes to remote
 */
export function usePushWorkspace() {
  return useMutation({
    mutationFn: ({ workspaceId, repoId, force }: { workspaceId: string; repoId: string; force?: boolean }) =>
      workspacesApi.push(workspaceId, { repo_id: repoId, force }),
  })
}

/**
 * Create a pull request from workspace
 */
export function useCreatePR() {
  return useMutation({
    mutationFn: ({
      workspaceId,
      repoId,
      title,
      body,
    }: {
      workspaceId: string
      repoId: string
      title: string
      body: string
    }) => workspacesApi.createPR(workspaceId, { repo_id: repoId, title, body }),
  })
}

/**
 * Start dev server for workspace
 */
export function useStartDevServer() {
  return useMutation({
    mutationFn: (workspaceId: string) => workspacesApi.startDevServer(workspaceId),
  })
}

/**
 * Fetch sessions for a workspace
 */
export function useWorkspaceSessions(workspaceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      workspaceId.value ? queryKeys.sessions.byWorkspace(workspaceId.value) : ['sessions', null]
    ),
    queryFn: () => {
      if (!workspaceId.value) throw new Error('Workspace ID is required')
      return sessionsApi.getByWorkspace(workspaceId.value)
    },
    enabled: computed(() => !!workspaceId.value),
  })
}

/**
 * Create a new session
 */
export function useCreateSession() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: { workspace_id: string; executor?: string }) => sessionsApi.create(data),
    onSuccess: (newSession) => {
      queryClient.setQueryData(queryKeys.sessions.detail(newSession.id), newSession)
      queryClient.invalidateQueries({
        queryKey: queryKeys.sessions.byWorkspace(newSession.workspace_id),
      })
    },
  })
}

/**
 * Send follow-up message to session
 */
export function useSessionFollowUp() {
  return useMutation({
    mutationFn: ({
      sessionId,
      prompt,
      variant,
    }: {
      sessionId: string
      prompt: string
      variant?: string | null
    }) => sessionsApi.followUp(sessionId, { prompt, variant }),
  })
}

/**
 * Workspace status helpers
 */
export function getWorkspaceStatusInfo(workspace: Workspace) {
  const isSetupComplete = !!workspace.setup_completed_at
  const hasContainer = !!workspace.container_ref

  if (workspace.archived) {
    return {
      label: '已归档',
      color: 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400',
      icon: 'M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4',
    }
  }

  if (!isSetupComplete) {
    return {
      label: '初始化中',
      color: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      icon: 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15',
    }
  }

  if (hasContainer) {
    return {
      label: '运行中',
      color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      icon: 'M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z',
    }
  }

  return {
    label: '就绪',
    color: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
  }
}
