<script setup lang="ts">
/**
 * TaskBoard - Kanban board for task management
 * Features:
 * - Real-time task updates via WebSocket
 * - Drag and drop between columns
 * - Batch task selection
 */
import { computed, toRef } from 'vue'
import { VueDraggable } from 'vue-draggable-plus'
import { useProjectsStore, useUiStore } from '@/stores'
import { useTasksStream } from '@/lib/websocket'
import { groupTasksByStatus, taskStatusInfo, useUpdateTask } from '@/composables/useTasks'
import TaskCard from './TaskCard.vue'
import Button from '@/components/ui/Button.vue'
import Loading from '@/components/ui/Loading.vue'
import type { Task, TaskStatus } from '@/types'

const projectsStore = useProjectsStore()
const uiStore = useUiStore()
const updateTaskMutation = useUpdateTask()

// Real-time task data via WebSocket
const projectId = toRef(projectsStore, 'currentProjectId')
const { tasks, status: wsStatus, error: wsError, reconnect } = useTasksStream(projectId)

// Group tasks by status
const tasksByStatus = computed(() => groupTasksByStatus(tasks.value))

// Column configuration
const columns: { status: TaskStatus; label: string }[] = [
  { status: 'todo', label: '待办' },
  { status: 'inprogress', label: '进行中' },
  { status: 'inreview', label: '待审核' },
  { status: 'done', label: '已完成' },
]

// Connection status
const isConnected = computed(() => wsStatus.value === 'OPEN')
const isConnecting = computed(() => wsStatus.value === 'CONNECTING')

// Get tasks for a column (mutable array for drag and drop)
const getColumnTasks = (status: TaskStatus) => {
  return tasksByStatus.value[status] || []
}

// Handle drag and drop
const handleDragEnd = async (evt: unknown) => {
  const event = evt as { item?: HTMLElement; to?: HTMLElement }
  const taskId = event.item?.dataset?.taskId
  const newStatus = event.to?.dataset?.status as TaskStatus

  if (!taskId || !newStatus) return

  const task = tasks.value[taskId]
  if (!task || task.status === newStatus) return

  // Update task status via API
  try {
    await updateTaskMutation.mutateAsync({
      taskId,
      data: { status: newStatus },
    })
  } catch (error) {
    console.error('Failed to update task status:', error)
    // The WebSocket will sync the correct state
  }
}

// Open task detail panel
const handleTaskClick = (task: Task) => {
  uiStore.openTaskPanel(task.id)
}

// Open edit dialog
const handleTaskEdit = (task: Task) => {
  uiStore.openDialog('editTask', task)
}

// Open delete confirmation
const handleTaskDelete = (task: Task) => {
  uiStore.openDialog('deleteTask', task)
}

// Open create task dialog
const handleCreateTask = (status?: TaskStatus) => {
  uiStore.openDialog('createTask', { status })
}

// Clear batch selection
const clearSelection = () => {
  uiStore.clearTaskSelection()
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border-normal">
      <div class="flex items-center gap-3">
        <h2 class="text-lg font-semibold text-text-primary">任务看板</h2>

        <!-- Connection Status -->
        <div class="flex items-center gap-1.5 text-xs">
          <span
            class="w-2 h-2 rounded-full"
            :class="{
              'bg-green-500': isConnected,
              'bg-yellow-500 animate-pulse': isConnecting,
              'bg-red-500': !isConnected && !isConnecting,
            }"
          />
          <span class="text-text-muted">
            {{ isConnected ? '已连接' : isConnecting ? '连接中...' : '未连接' }}
          </span>
          <button
            v-if="!isConnected && !isConnecting"
            type="button"
            class="text-brand hover:underline"
            @click="reconnect"
          >
            重连
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- Batch Selection Info -->
        <div v-if="uiStore.hasSelectedTasks" class="flex items-center gap-2 mr-2">
          <span class="text-sm text-text-muted">
            已选择 {{ uiStore.selectedTaskCount }} 个任务
          </span>
          <button
            type="button"
            class="text-xs text-text-muted hover:text-text-primary"
            @click="clearSelection"
          >
            清除
          </button>
        </div>

        <!-- Add Task Button -->
        <Button variant="primary" size="sm" @click="handleCreateTask()">
          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          新建任务
        </Button>
      </div>
    </div>

    <!-- Error State -->
    <div v-if="wsError" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="text-red-500 mb-4">
          <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-secondary">{{ wsError.message || '连接失败' }}</p>
        </div>
        <Button variant="secondary" @click="reconnect">重新连接</Button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-else-if="isConnecting && Object.keys(tasks).length === 0" class="flex-1 flex items-center justify-center">
      <Loading />
    </div>

    <!-- No Project Selected -->
    <div v-else-if="!projectsStore.currentProjectId" class="flex-1 flex items-center justify-center">
      <div class="text-center text-text-muted">
        <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <p>请先选择一个项目</p>
      </div>
    </div>

    <!-- Kanban Board -->
    <div v-else class="flex-1 overflow-x-auto p-4">
      <div class="flex gap-4 h-full min-w-max">
        <!-- Columns -->
        <div
          v-for="column in columns"
          :key="column.status"
          class="flex flex-col w-72 bg-bg-secondary rounded-lg"
        >
          <!-- Column Header -->
          <div class="flex items-center justify-between px-3 py-2 border-b border-border-normal">
            <div class="flex items-center gap-2">
              <span
                class="w-3 h-3 rounded-full"
                :class="taskStatusInfo[column.status].color.split(' ')[0]"
              />
              <h3 class="font-medium text-text-primary">{{ column.label }}</h3>
              <span class="text-xs text-text-muted bg-bg-hover px-1.5 py-0.5 rounded">
                {{ getColumnTasks(column.status).length }}
              </span>
            </div>
            <button
              type="button"
              class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
              title="添加任务"
              @click="handleCreateTask(column.status)"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
            </button>
          </div>

          <!-- Task List -->
          <VueDraggable
            :model-value="getColumnTasks(column.status)"
            group="tasks"
            :data-status="column.status"
            class="flex-1 p-2 space-y-2 overflow-y-auto min-h-[200px]"
            item-key="id"
            ghost-class="opacity-50"
            drag-class="shadow-lg"
            @end="handleDragEnd"
          >
            <template #item="{ element: task }">
              <TaskCard
                :task="task"
                :data-task-id="task.id"
                :selected="uiStore.selectedTaskId === task.id"
                draggable
                @click="handleTaskClick(task)"
                @edit="handleTaskEdit(task)"
                @delete="handleTaskDelete(task)"
              />
            </template>
          </VueDraggable>
        </div>
      </div>
    </div>
  </div>
</template>
