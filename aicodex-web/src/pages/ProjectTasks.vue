<script setup lang="ts">
/**
 * ProjectTasks.vue - Task kanban board for a specific project
 */
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProject } from '@/composables/useProjects'
import { useTasks, groupTasksByStatus } from '@/composables/useTasks'
import { useUiStore } from '@/stores'
import { Button, Loading } from '@/components/ui'
import { PageHeader } from '@/components/layout'
import { TaskBoard, TaskFilter, TaskFormDialog, TaskDetailPanel } from '@/components/tasks'
import { ProjectFormDialog } from '@/components/projects'
import type { TaskStatus } from '@/types'

const route = useRoute()
const router = useRouter()
const uiStore = useUiStore()

// Get project ID from route
const projectId = computed(() => route.params.projectId as string)
const projectIdRef = ref(projectId.value)
watch(projectId, (id) => {
  projectIdRef.value = id
})

// Fetch project and tasks
const { data: project, isLoading: projectLoading, error: projectError } = useProject(projectIdRef)
const { data: tasks, isLoading: tasksLoading, error: tasksError, refetch: refetchTasks } = useTasks(projectIdRef)

// Filter state
const searchQuery = ref('')
const statusFilter = ref<TaskStatus[]>([])
const tagFilter = ref<string[]>([])

// Computed
const isLoading = computed(() => projectLoading.value || tasksLoading.value)
const error = computed(() => projectError.value || tasksError.value)

const filteredTasks = computed(() => {
  if (!tasks.value) return []

  let filtered = [...tasks.value]

  // Search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(task =>
      task.title.toLowerCase().includes(query) ||
      task.description?.toLowerCase().includes(query)
    )
  }

  // Status filter
  if (statusFilter.value.length > 0) {
    filtered = filtered.filter(task => statusFilter.value.includes(task.status))
  }

  return filtered
})

const groupedTasks = computed(() => groupTasksByStatus(filteredTasks.value))

// Available tags - not currently used since Task type doesn't have tags
const availableTags = computed<string[]>(() => [])

// Methods
const handleBack = () => {
  router.push('/tasks')
}

const handleCreateTask = () => {
  uiStore.openDialog('createTask', { projectId: projectId.value })
}

const handleEditProject = () => {
  uiStore.openDialog('editProject', { projectId: projectId.value })
}

const handleSearch = (query: string) => {
  searchQuery.value = query
}

const handleFilterStatus = (statuses: TaskStatus[]) => {
  statusFilter.value = statuses
}

const handleFilterTags = (tags: string[]) => {
  tagFilter.value = tags
}

const handleClearFilters = () => {
  searchQuery.value = ''
  statusFilter.value = []
  tagFilter.value = []
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <PageHeader :title="project?.name || '加载中...'">
      <template #prefix>
        <button
          type="button"
          class="p-1.5 -ml-1.5 mr-2 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded-lg transition-colors"
          @click="handleBack"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
      </template>
      <template #actions>
        <Button variant="ghost" size="sm" @click="handleEditProject">
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </Button>
        <Button variant="secondary" size="sm" @click="refetchTasks()">
          <svg class="h-4 w-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          刷新
        </Button>
        <Button variant="primary" @click="handleCreateTask">
          <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          新建任务
        </Button>
      </template>
    </PageHeader>

    <!-- Filter Bar -->
    <div class="px-4 py-3 border-b border-border-normal bg-bg-secondary">
      <TaskFilter
        :available-tags="availableTags"
        @search="handleSearch"
        @filter-status="handleFilterStatus"
        @filter-tags="handleFilterTags"
        @clear="handleClearFilters"
      />
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-hidden">
      <!-- Loading State -->
      <div v-if="isLoading" class="flex items-center justify-center h-full">
        <Loading size="lg" />
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="flex items-center justify-center h-full">
        <div class="text-center">
          <div class="text-red-500 mb-4">
            <svg class="w-16 h-16 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <p class="text-lg font-medium">加载失败</p>
            <p class="text-sm text-text-muted mt-1">{{ error.message }}</p>
          </div>
          <div class="flex gap-3 justify-center">
            <Button variant="secondary" @click="handleBack">返回项目列表</Button>
            <Button variant="primary" @click="refetchTasks()">重试</Button>
          </div>
        </div>
      </div>

      <!-- Task Board -->
      <TaskBoard
        v-else
        :project-id="projectId"
        :tasks="groupedTasks"
        class="h-full"
      />
    </div>

    <!-- Dialogs -->
    <TaskFormDialog />
    <TaskDetailPanel />
    <ProjectFormDialog />
  </div>
</template>
