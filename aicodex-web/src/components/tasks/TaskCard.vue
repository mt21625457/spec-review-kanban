<script setup lang="ts">
/**
 * TaskCard - Card component for displaying a single task
 */
import { computed } from 'vue'
import { useUiStore } from '@/stores'
import { taskStatusInfo } from '@/composables/useTasks'
import Badge from '@/components/ui/Badge.vue'
import type { Task } from '@/types'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

const props = defineProps<{
  task: Task
  selected?: boolean
  draggable?: boolean
}>()

const emit = defineEmits<{
  click: [task: Task]
  edit: [task: Task]
  delete: [task: Task]
}>()

const uiStore = useUiStore()

const statusInfo = computed(() => taskStatusInfo[props.task.status])

const formattedDate = computed(() => {
  try {
    return formatDistanceToNow(new Date(props.task.updated_at), {
      addSuffix: true,
      locale: zhCN,
    })
  } catch {
    return props.task.updated_at
  }
})

const isInBatchSelection = computed(() =>
  uiStore.selectedTaskIds.has(props.task.id)
)

const handleClick = () => {
  emit('click', props.task)
}

const handleEdit = (e: Event) => {
  e.stopPropagation()
  emit('edit', props.task)
}

const handleDelete = (e: Event) => {
  e.stopPropagation()
  emit('delete', props.task)
}

const toggleSelection = (e: Event) => {
  e.stopPropagation()
  uiStore.toggleTaskSelection(props.task.id)
}
</script>

<template>
  <div
    class="group relative bg-bg-primary border rounded-lg p-3 cursor-pointer transition-all hover:shadow-md"
    :class="{
      'border-brand ring-2 ring-brand/20': selected,
      'border-border-normal hover:border-border-hover': !selected,
      'ring-2 ring-blue-500/30 border-blue-500': isInBatchSelection,
      'cursor-move': draggable,
    }"
    @click="handleClick"
  >
    <!-- Batch Selection Checkbox -->
    <div
      class="absolute -left-2 -top-2 opacity-0 group-hover:opacity-100 transition-opacity"
      :class="{ 'opacity-100': isInBatchSelection }"
    >
      <button
        type="button"
        class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors"
        :class="
          isInBatchSelection
            ? 'bg-blue-500 border-blue-500 text-white'
            : 'bg-bg-primary border-border-normal hover:border-blue-500'
        "
        @click="toggleSelection"
      >
        <svg v-if="isInBatchSelection" class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      </button>
    </div>

    <!-- Header -->
    <div class="flex items-start justify-between gap-2 mb-2">
      <h4 class="text-sm font-medium text-text-primary line-clamp-2 flex-1">
        {{ task.title }}
      </h4>

      <!-- Actions -->
      <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
        <button
          type="button"
          class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
          title="编辑"
          @click="handleEdit"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        </button>
        <button
          type="button"
          class="p-1 text-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
          title="删除"
          @click="handleDelete"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Description Preview -->
    <p v-if="task.description" class="text-xs text-text-muted line-clamp-2 mb-2">
      {{ task.description }}
    </p>

    <!-- Footer -->
    <div class="flex items-center justify-between">
      <!-- Status Badge -->
      <Badge :class="statusInfo.color" size="sm">
        <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="statusInfo.icon" />
        </svg>
        {{ statusInfo.label }}
      </Badge>

      <!-- Time -->
      <span class="text-xs text-text-muted">{{ formattedDate }}</span>
    </div>

    <!-- Shared Task Indicator -->
    <div
      v-if="task.shared_task_id"
      class="absolute top-2 right-2 text-brand"
      title="已共享"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
      </svg>
    </div>
  </div>
</template>
