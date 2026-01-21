<script setup lang="ts">
/**
 * TaskDetailPanel - Sliding panel for task details and workspaces
 */
import { computed, toRef } from 'vue'
import { useUiStore } from '@/stores'
import { useTask, taskStatusInfo, useShareTask } from '@/composables/useTasks'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import Loading from '@/components/ui/Loading.vue'
import { formatDistanceToNow, format } from 'date-fns'
import { zhCN } from 'date-fns/locale'

const uiStore = useUiStore()

// Fetch task details
const taskId = toRef(uiStore, 'selectedTaskId')
const { data: task, isLoading, error, refetch } = useTask(taskId)

const shareMutation = useShareTask()

// Computed
const isOpen = computed(() => uiStore.activePanelType === 'task' && !!taskId.value)
const statusInfo = computed(() => task.value ? taskStatusInfo[task.value.status] : null)

const formatDate = (dateStr: string | null) => {
  if (!dateStr) return '-'
  try {
    return format(new Date(dateStr), 'yyyy-MM-dd HH:mm', { locale: zhCN })
  } catch {
    return dateStr
  }
}

const relativeTime = (dateStr: string | null) => {
  if (!dateStr) return '-'
  try {
    return formatDistanceToNow(new Date(dateStr), { addSuffix: true, locale: zhCN })
  } catch {
    return dateStr
  }
}

// Methods
const handleClose = () => {
  uiStore.closePanel()
}

const handleEdit = () => {
  if (task.value) {
    uiStore.openDialog('editTask', task.value)
  }
}

const handleDelete = () => {
  if (task.value) {
    uiStore.openDialog('deleteTask', task.value)
  }
}

const handleShare = async () => {
  if (!task.value) return
  try {
    await shareMutation.mutateAsync(task.value.id)
    uiStore.openDialog('shareTask', task.value)
  } catch (error) {
    console.error('Failed to share task:', error)
  }
}

const handleStartWork = () => {
  // Open create workspace dialog or directly start
  if (task.value) {
    uiStore.openDialog('createTask', { taskId: task.value.id, action: 'start' })
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="translate-x-full"
      enter-to-class="translate-x-0"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="translate-x-0"
      leave-to-class="translate-x-full"
    >
      <div
        v-if="isOpen"
        class="fixed inset-y-0 right-0 w-full max-w-lg bg-bg-primary border-l border-border-normal shadow-xl z-40 flex flex-col"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3 border-b border-border-normal">
          <h2 class="text-lg font-semibold text-text-primary">任务详情</h2>
          <button
            type="button"
            class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
            @click="handleClose"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto">
          <!-- Loading State -->
          <div v-if="isLoading" class="flex items-center justify-center h-64">
            <Loading />
          </div>

          <!-- Error State -->
          <div v-else-if="error" class="p-4 text-center">
            <div class="text-red-500 mb-4">
              <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <p class="text-text-secondary">加载任务失败</p>
            </div>
            <Button variant="secondary" @click="refetch">重试</Button>
          </div>

          <!-- Task Details -->
          <div v-else-if="task" class="p-4 space-y-6">
            <!-- Title & Actions -->
            <div>
              <div class="flex items-start justify-between gap-3 mb-2">
                <h3 class="text-xl font-semibold text-text-primary">{{ task.title }}</h3>
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    class="p-1.5 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
                    title="编辑"
                    @click="handleEdit"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-text-muted hover:text-brand hover:bg-brand/10 rounded transition-colors"
                    title="分享"
                    @click="handleShare"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
                    </svg>
                  </button>
                  <button
                    type="button"
                    class="p-1.5 text-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                    title="删除"
                    @click="handleDelete"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Status Badge -->
              <Badge v-if="statusInfo" :class="statusInfo.color">
                <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="statusInfo.icon" />
                </svg>
                {{ statusInfo.label }}
              </Badge>
            </div>

            <!-- Description -->
            <div v-if="task.description">
              <h4 class="text-sm font-medium text-text-secondary mb-2">描述</h4>
              <p class="text-text-primary whitespace-pre-wrap">{{ task.description }}</p>
            </div>

            <!-- Metadata -->
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-text-muted">创建时间</span>
                <p class="text-text-primary" :title="formatDate(task.created_at)">
                  {{ relativeTime(task.created_at) }}
                </p>
              </div>
              <div>
                <span class="text-text-muted">更新时间</span>
                <p class="text-text-primary" :title="formatDate(task.updated_at)">
                  {{ relativeTime(task.updated_at) }}
                </p>
              </div>
            </div>

            <!-- Shared Task Info -->
            <div v-if="task.shared_task_id" class="p-3 bg-brand/10 rounded-lg">
              <div class="flex items-center gap-2 text-brand">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
                </svg>
                <span class="text-sm font-medium">已共享的任务</span>
              </div>
            </div>

            <!-- Workspaces Section -->
            <div>
              <div class="flex items-center justify-between mb-3">
                <h4 class="text-sm font-medium text-text-secondary">工作区</h4>
                <Button variant="primary" size="sm" @click="handleStartWork">
                  <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  启动工作
                </Button>
              </div>

              <!-- Empty State -->
              <div class="text-center py-8 text-text-muted">
                <svg class="w-10 h-10 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <p class="text-sm">暂无工作区</p>
                <p class="text-xs mt-1">点击"启动工作"创建一个新的工作区</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Backdrop -->
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 bg-black/30 z-30"
        @click="handleClose"
      />
    </Transition>
  </Teleport>
</template>
