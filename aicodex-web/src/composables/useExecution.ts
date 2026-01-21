/**
 * useExecution - Vue Query hooks for execution process management
 */
import { computed, type Ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { executionProcessesApi } from '@/lib/api'
import { queryKeys } from '@/lib/queryClient'
import type { ExecutionProcess, ExecutionProcessStatus } from '@/types'

/**
 * Fetch execution process by ID
 */
export function useExecutionProcess(processId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      processId.value ? queryKeys.executionProcesses.detail(processId.value) : ['executionProcesses', null]
    ),
    queryFn: () => {
      if (!processId.value) throw new Error('Process ID is required')
      return executionProcessesApi.get(processId.value)
    },
    enabled: computed(() => !!processId.value),
    refetchInterval: (query) => {
      // Auto-refetch while running
      const data = query.state.data as ExecutionProcess | undefined
      return data?.status === 'running' ? 2000 : false
    },
  })
}

/**
 * Fetch execution process repo states
 */
export function useExecutionProcessRepoStates(processId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      processId.value ? queryKeys.executionProcesses.repoStates(processId.value) : ['executionProcesses', null, 'repoStates']
    ),
    queryFn: () => {
      if (!processId.value) throw new Error('Process ID is required')
      return executionProcessesApi.getRepoStates(processId.value)
    },
    enabled: computed(() => !!processId.value),
  })
}

/**
 * Stop an execution process
 */
export function useStopExecutionProcess() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (processId: string) => executionProcessesApi.stop(processId),
    onSuccess: (_, processId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.executionProcesses.detail(processId) })
    },
  })
}

/**
 * Execution status display info
 */
export const executionStatusInfo: Record<ExecutionProcessStatus, { label: string; color: string; icon: string }> = {
  running: {
    label: '运行中',
    color: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    icon: 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15',
  },
  completed: {
    label: '已完成',
    color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    icon: 'M5 13l4 4L19 7',
  },
  failed: {
    label: '失败',
    color: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    icon: 'M6 18L18 6M6 6l12 12',
  },
  killed: {
    label: '已终止',
    color: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200',
    icon: 'M21 12a9 9 0 11-18 0 9 9 0 0118 0z M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z',
  },
}

/**
 * Run reason display info
 */
export const runReasonInfo: Record<string, { label: string; description: string }> = {
  setupscript: {
    label: '初始化脚本',
    description: '执行工作区初始化脚本',
  },
  cleanupscript: {
    label: '清理脚本',
    description: '执行工作区清理脚本',
  },
  codingagent: {
    label: 'AI 编码',
    description: 'AI 代理执行编码任务',
  },
  devserver: {
    label: '开发服务器',
    description: '运行开发服务器',
  },
}

/**
 * Format cost to display string
 */
export function formatCost(costUsd: string | null): string {
  if (!costUsd) return '-'
  const cost = parseFloat(costUsd)
  if (isNaN(cost)) return '-'
  return `$${cost.toFixed(4)}`
}

/**
 * Format token count
 */
export function formatTokens(tokens: number | null): string {
  if (tokens === null) return '-'
  if (tokens >= 1000000) {
    return `${(tokens / 1000000).toFixed(1)}M`
  }
  if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(1)}K`
  }
  return tokens.toString()
}
