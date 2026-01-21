/**
 * useTasks - Vue Query hooks for task management
 */
import { computed, type Ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { tasksApi } from '@/lib/api'
import { queryKeys } from '@/lib/queryClient'
import type { Task, CreateTask, UpdateTask, TaskStatus, TaskWithAttemptStatus } from '@/types'

/**
 * Fetch all tasks for a project
 */
export function useTasks(projectId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      projectId.value ? queryKeys.tasks.byProject(projectId.value) : ['tasks', null]
    ),
    queryFn: () => {
      if (!projectId.value) throw new Error('Project ID is required')
      return tasksApi.getByProject(projectId.value)
    },
    enabled: computed(() => !!projectId.value),
  })
}

/**
 * Fetch a single task by ID
 */
export function useTask(taskId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      taskId.value ? queryKeys.tasks.detail(taskId.value) : ['tasks', null]
    ),
    queryFn: () => {
      if (!taskId.value) throw new Error('Task ID is required')
      return tasksApi.getById(taskId.value)
    },
    enabled: computed(() => !!taskId.value),
  })
}

/**
 * Create a new task
 */
export function useCreateTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateTask) => tasksApi.create(data),
    onSuccess: (newTask) => {
      // WebSocket will handle the update via tasks stream
      // But we can also update the cache
      queryClient.setQueryData(queryKeys.tasks.detail(newTask.id), newTask)
    },
  })
}

/**
 * Create and start a new task with an agent
 */
export function useCreateAndStartTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateTask & { executor: string }) => tasksApi.createAndStart(data),
    onSuccess: (result) => {
      queryClient.setQueryData(queryKeys.tasks.detail(result.id), result)
    },
  })
}

/**
 * Update an existing task
 */
export function useUpdateTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ taskId, data }: { taskId: string; data: UpdateTask }) =>
      tasksApi.update(taskId, data),
    onSuccess: (updatedTask, { taskId }) => {
      queryClient.setQueryData(queryKeys.tasks.detail(taskId), updatedTask)
    },
  })
}

/**
 * Delete a task
 */
export function useDeleteTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (taskId: string) => tasksApi.delete(taskId),
    onSuccess: (_, taskId) => {
      queryClient.removeQueries({ queryKey: queryKeys.tasks.detail(taskId) })
    },
  })
}

/**
 * Share a task
 */
export function useShareTask() {
  return useMutation({
    mutationFn: (taskId: string) => tasksApi.share(taskId),
  })
}

/**
 * Unshare a task
 */
export function useUnshareTask() {
  return useMutation({
    mutationFn: (sharedTaskId: string) => tasksApi.unshare(sharedTaskId),
  })
}

/**
 * Get shared task info
 */
export function useSharedTask(sharedTaskId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      sharedTaskId.value ? queryKeys.tasks.shared(sharedTaskId.value) : ['sharedTasks', null]
    ),
    queryFn: () => {
      if (!sharedTaskId.value) throw new Error('Shared Task ID is required')
      return tasksApi.getSharedTask(sharedTaskId.value)
    },
    enabled: computed(() => !!sharedTaskId.value),
  })
}

/**
 * Link a shared task to local project
 */
export function useLinkSharedTask() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: { shared_task_id: string; project_id: string }) =>
      tasksApi.linkSharedTask(data),
    onSuccess: (newTask) => {
      queryClient.setQueryData(queryKeys.tasks.detail(newTask.id), newTask)
    },
  })
}

/**
 * Helper to group tasks by status
 * Accepts either an array of tasks or a Record<string, Task>
 */
export function groupTasksByStatus(tasks: Task[] | TaskWithAttemptStatus[] | Record<string, Task>) {
  const grouped: Record<TaskStatus, Task[]> = {
    todo: [],
    inprogress: [],
    inreview: [],
    done: [],
    cancelled: [],
  }

  // Handle both array and record formats
  const taskArray = Array.isArray(tasks) ? tasks : Object.values(tasks)

  for (const task of taskArray) {
    if (grouped[task.status]) {
      grouped[task.status].push(task)
    }
  }

  // Sort by updated_at descending within each group
  for (const status of Object.keys(grouped) as TaskStatus[]) {
    grouped[status].sort(
      (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
    )
  }

  return grouped
}

/**
 * Status display info
 */
export const taskStatusInfo: Record<TaskStatus, { label: string; color: string; icon: string }> = {
  todo: {
    label: '待办',
    color: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200',
    icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2',
  },
  inprogress: {
    label: '进行中',
    color: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    icon: 'M13 10V3L4 14h7v7l9-11h-7z',
  },
  inreview: {
    label: '待审核',
    color: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
    icon: 'M15 12a3 3 0 11-6 0 3 3 0 016 0z M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z',
  },
  done: {
    label: '已完成',
    color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    icon: 'M5 13l4 4L19 7',
  },
  cancelled: {
    label: '已取消',
    color: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    icon: 'M6 18L18 6M6 6l12 12',
  },
}
