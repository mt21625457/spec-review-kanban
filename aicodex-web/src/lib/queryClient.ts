/**
 * Vue Query Configuration
 * Server state management with caching, background refetching, and optimistic updates
 */
import { QueryClient } from '@tanstack/vue-query'

// Create a client with default options
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Stale time: 30 seconds before data is considered stale
      staleTime: 30 * 1000,
      // Cache time: 5 minutes before unused data is garbage collected
      gcTime: 5 * 60 * 1000,
      // Retry failed requests up to 3 times with exponential backoff
      retry: 3,
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
      // Refetch on window focus for fresh data
      refetchOnWindowFocus: true,
      // Don't refetch on mount if data is fresh
      refetchOnMount: true,
    },
    mutations: {
      // Retry mutations once on failure
      retry: 1,
    },
  },
})

// Query keys factory for type-safe query key management
export const queryKeys = {
  // Projects
  projects: {
    all: ['projects'] as const,
    detail: (id: string) => ['projects', id] as const,
    repositories: (projectId: string) => ['projects', projectId, 'repositories'] as const,
  },

  // Tasks
  tasks: {
    all: ['tasks'] as const,
    byProject: (projectId: string) => ['tasks', { projectId }] as const,
    detail: (id: string) => ['tasks', id] as const,
    shared: (sharedTaskId: string) => ['sharedTasks', sharedTaskId] as const,
  },

  // Tags
  tags: {
    all: ['tags'] as const,
    search: (query: string) => ['tags', { search: query }] as const,
  },

  // Workspaces (Task Attempts)
  workspaces: {
    all: ['workspaces'] as const,
    byTask: (taskId: string) => ['workspaces', { taskId }] as const,
    detail: (id: string) => ['workspaces', id] as const,
    repos: (workspaceId: string) => ['workspaces', workspaceId, 'repos'] as const,
    branchStatus: (workspaceId: string) => ['workspaces', workspaceId, 'branchStatus'] as const,
  },

  // Sessions
  sessions: {
    all: ['sessions'] as const,
    byWorkspace: (workspaceId: string) => ['sessions', { workspaceId }] as const,
    detail: (id: string) => ['sessions', id] as const,
  },

  // Execution Processes
  executionProcesses: {
    detail: (id: string) => ['executionProcesses', id] as const,
    repoStates: (id: string) => ['executionProcesses', id, 'repoStates'] as const,
  },

  // Repositories
  repos: {
    all: ['repos'] as const,
    detail: (id: string) => ['repos', id] as const,
    branches: (id: string) => ['repos', id, 'branches'] as const,
  },

  // System
  system: {
    info: ['system', 'info'] as const,
    agentAvailability: (agent: string) => ['system', 'agentAvailability', agent] as const,
  },

  // Authentication
  auth: {
    me: ['auth', 'me'] as const,
  },

  // User instances
  myInstances: {
    all: ['myInstances'] as const,
    current: ['myInstances', 'current'] as const,
    health: ['myInstances', 'health'] as const,
  },
} as const

// Type for query key inference
export type QueryKeys = typeof queryKeys
