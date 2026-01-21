<script setup lang="ts">
/**
 * WorkspaceCard - Card component for displaying a single workspace
 */
import { computed } from 'vue'
import { getWorkspaceStatusInfo } from '@/composables/useWorkspaces'
import Badge from '@/components/ui/Badge.vue'
import type { Workspace } from '@/types'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

const props = defineProps<{
  workspace: Workspace
  selected?: boolean
}>()

const emit = defineEmits<{
  click: [workspace: Workspace]
  viewDiffs: [workspace: Workspace]
  viewLogs: [workspace: Workspace]
  stop: [workspace: Workspace]
  archive: [workspace: Workspace]
  delete: [workspace: Workspace]
}>()

const statusInfo = computed(() => getWorkspaceStatusInfo(props.workspace))

const formattedDate = computed(() => {
  try {
    return formatDistanceToNow(new Date(props.workspace.created_at), {
      addSuffix: true,
      locale: zhCN,
    })
  } catch {
    return props.workspace.created_at
  }
})

const displayName = computed(() => {
  return props.workspace.name || `工作区 ${props.workspace.branch}`
})

const handleClick = () => {
  emit('click', props.workspace)
}

const handleViewDiffs = (e: Event) => {
  e.stopPropagation()
  emit('viewDiffs', props.workspace)
}

const handleViewLogs = (e: Event) => {
  e.stopPropagation()
  emit('viewLogs', props.workspace)
}

const handleStop = (e: Event) => {
  e.stopPropagation()
  emit('stop', props.workspace)
}

const handleArchive = (e: Event) => {
  e.stopPropagation()
  emit('archive', props.workspace)
}

const handleDelete = (e: Event) => {
  e.stopPropagation()
  emit('delete', props.workspace)
}
</script>

<template>
  <div
    class="group relative bg-bg-primary border rounded-lg p-4 cursor-pointer transition-all hover:shadow-md"
    :class="{
      'border-brand ring-2 ring-brand/20': selected,
      'border-border-normal hover:border-border-hover': !selected,
      'opacity-60': workspace.archived,
    }"
    @click="handleClick"
  >
    <!-- Header -->
    <div class="flex items-start justify-between gap-2 mb-3">
      <div class="min-w-0 flex-1">
        <h4 class="text-sm font-medium text-text-primary truncate">
          {{ displayName }}
        </h4>
        <p class="text-xs text-text-muted truncate mt-0.5">
          {{ workspace.branch }}
        </p>
      </div>

      <!-- Status Badge -->
      <Badge :class="statusInfo.color" size="sm">
        <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="statusInfo.icon" />
        </svg>
        {{ statusInfo.label }}
      </Badge>
    </div>

    <!-- Meta Info -->
    <div class="flex items-center gap-4 text-xs text-text-muted mb-3">
      <div class="flex items-center gap-1">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span>{{ formattedDate }}</span>
      </div>

      <div v-if="workspace.agent_working_dir" class="flex items-center gap-1 truncate">
        <svg class="w-3.5 h-3.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <span class="truncate">{{ workspace.agent_working_dir }}</span>
      </div>
    </div>

    <!-- Pinned Indicator -->
    <div
      v-if="workspace.pinned"
      class="absolute top-2 right-2 text-yellow-500"
      title="已固定"
    >
      <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
      </svg>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-1 pt-2 border-t border-border-normal">
      <!-- View Diffs -->
      <button
        type="button"
        class="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 text-xs text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
        title="查看变更"
        @click="handleViewDiffs"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <span>变更</span>
      </button>

      <!-- View Logs -->
      <button
        type="button"
        class="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 text-xs text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
        title="查看日志"
        @click="handleViewLogs"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16m-7 6h7" />
        </svg>
        <span>日志</span>
      </button>

      <!-- Stop (if running) -->
      <button
        v-if="workspace.container_ref && !workspace.archived"
        type="button"
        class="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 text-xs text-yellow-600 hover:text-yellow-700 hover:bg-yellow-50 dark:hover:bg-yellow-900/20 rounded transition-colors"
        title="停止"
        @click="handleStop"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
        </svg>
        <span>停止</span>
      </button>

      <!-- Archive -->
      <button
        v-if="!workspace.archived"
        type="button"
        class="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 text-xs text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
        title="归档"
        @click="handleArchive"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
        </svg>
        <span>归档</span>
      </button>

      <!-- Delete -->
      <button
        type="button"
        class="p-1.5 text-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
        title="删除"
        @click="handleDelete"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  </div>
</template>
