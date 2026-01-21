<script setup lang="ts">
/**
 * WorkspaceList - List of workspaces for a task
 */
import { computed, ref, toRef } from 'vue'
import { useUiStore } from '@/stores'
import {
  useWorkspaces,
  useStopWorkspace,
  useUpdateWorkspace,
  useDeleteWorkspace,
} from '@/composables/useWorkspaces'
import WorkspaceCard from './WorkspaceCard.vue'
import Button from '@/components/ui/Button.vue'
import Loading from '@/components/ui/Loading.vue'
import type { Workspace } from '@/types'

const props = defineProps<{
  taskId: string
}>()

const emit = defineEmits<{
  createWorkspace: []
  selectWorkspace: [workspace: Workspace]
}>()

const uiStore = useUiStore()

// Fetch workspaces
const taskIdRef = toRef(props, 'taskId')
const { data: workspaces, isLoading, error, refetch } = useWorkspaces(taskIdRef)

// Mutations
const stopMutation = useStopWorkspace()
const updateMutation = useUpdateWorkspace()
const deleteMutation = useDeleteWorkspace()

// Filter state
const showArchived = ref(false)

// Computed
const filteredWorkspaces = computed(() => {
  if (!workspaces.value) return []

  let filtered = [...workspaces.value]

  if (!showArchived.value) {
    filtered = filtered.filter((w) => !w.archived)
  }

  // Sort: pinned first, then by created_at desc
  return filtered.sort((a, b) => {
    if (a.pinned && !b.pinned) return -1
    if (!a.pinned && b.pinned) return 1
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  })
})

const archivedCount = computed(() =>
  workspaces.value?.filter((w) => w.archived).length || 0
)

// Handlers
const handleWorkspaceClick = (workspace: Workspace) => {
  emit('selectWorkspace', workspace)
  uiStore.openAttemptPanel(workspace.id)
}

const handleViewDiffs = (workspace: Workspace) => {
  uiStore.openDiffsPanel(workspace.id)
}

const handleViewLogs = (workspace: Workspace) => {
  uiStore.openAttemptPanel(workspace.id)
}

const handleStop = async (workspace: Workspace) => {
  try {
    await stopMutation.mutateAsync(workspace.id)
  } catch (error) {
    console.error('Failed to stop workspace:', error)
  }
}

const handleArchive = async (workspace: Workspace) => {
  try {
    await updateMutation.mutateAsync({
      workspaceId: workspace.id,
      data: { archived: true },
    })
  } catch (error) {
    console.error('Failed to archive workspace:', error)
  }
}

const handleDelete = async (workspace: Workspace) => {
  if (!confirm(`确定要删除工作区 "${workspace.name || workspace.branch}" 吗？`)) {
    return
  }
  try {
    await deleteMutation.mutateAsync(workspace.id)
    // Close panel if viewing deleted workspace
    if (uiStore.selectedAttemptId === workspace.id) {
      uiStore.closePanel()
    }
  } catch (error) {
    console.error('Failed to delete workspace:', error)
  }
}

const handleCreate = () => {
  emit('createWorkspace')
}
</script>

<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <h3 class="text-base font-medium text-text-primary">工作区</h3>
        <span class="text-xs text-text-muted bg-bg-secondary px-1.5 py-0.5 rounded">
          {{ filteredWorkspaces.length }}
        </span>
      </div>

      <div class="flex items-center gap-2">
        <!-- Show Archived Toggle -->
        <label v-if="archivedCount > 0" class="flex items-center gap-1.5 text-xs text-text-muted cursor-pointer">
          <input
            v-model="showArchived"
            type="checkbox"
            class="rounded border-border-normal text-brand focus:ring-brand"
          />
          <span>显示已归档 ({{ archivedCount }})</span>
        </label>

        <!-- Create Button -->
        <Button variant="primary" size="sm" @click="handleCreate">
          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          新建
        </Button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="flex justify-center py-8">
      <Loading />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-center py-8">
      <div class="text-red-500 mb-4">
        <svg class="w-10 h-10 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="text-sm text-text-secondary">加载工作区失败</p>
      </div>
      <Button variant="secondary" size="sm" @click="refetch">重试</Button>
    </div>

    <!-- Empty State -->
    <div v-else-if="!filteredWorkspaces.length" class="text-center py-8">
      <div class="text-text-muted mb-4">
        <svg class="w-10 h-10 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <p class="text-sm">暂无工作区</p>
        <p class="text-xs mt-1">创建一个工作区开始执行任务</p>
      </div>
      <Button variant="primary" size="sm" @click="handleCreate">
        创建工作区
      </Button>
    </div>

    <!-- Workspace Grid -->
    <div v-else class="grid grid-cols-1 gap-3">
      <WorkspaceCard
        v-for="workspace in filteredWorkspaces"
        :key="workspace.id"
        :workspace="workspace"
        :selected="uiStore.selectedAttemptId === workspace.id"
        @click="handleWorkspaceClick"
        @view-diffs="handleViewDiffs"
        @view-logs="handleViewLogs"
        @stop="handleStop"
        @archive="handleArchive"
        @delete="handleDelete"
      />
    </div>
  </div>
</template>
