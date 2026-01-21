<script setup lang="ts">
/**
 * Tasks.vue - Project list page for task management
 * Users select a project to view its task kanban board
 */
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useProjects } from '@/composables/useProjects'
import { useUiStore } from '@/stores'
import { Card, Button, Loading } from '@/components/ui'
import { PageHeader } from '@/components/layout'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

const { t } = useI18n()
const router = useRouter()
const uiStore = useUiStore()

// Fetch projects
const { data: projects, isLoading, error, refetch } = useProjects()

// Computed
const sortedProjects = computed(() => {
  if (!projects.value) return []
  return [...projects.value].sort((a, b) => {
    // Sort by updated_at descending
    return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
  })
})

// Methods
const handleProjectClick = (projectId: string) => {
  router.push(`/tasks/${projectId}`)
}

const handleCreateProject = () => {
  uiStore.openDialog('createProject')
}

const formatDate = (dateString: string) => {
  try {
    return formatDistanceToNow(new Date(dateString), { addSuffix: true, locale: zhCN })
  } catch {
    return dateString
  }
}
</script>

<template>
  <div>
    <PageHeader :title="t('nav.tasks')">
      <template #actions>
        <Button variant="secondary" size="sm" @click="refetch()">
          <svg class="h-4 w-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          刷新
        </Button>
        <Button variant="primary" @click="handleCreateProject">
          <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          新建项目
        </Button>
      </template>
    </PageHeader>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex items-center justify-center py-20">
      <Loading size="lg" />
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="text-center py-20">
      <div class="text-red-500 mb-4">
        <svg class="w-16 h-16 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="text-lg font-medium">加载项目失败</p>
        <p class="text-sm text-text-muted mt-1">{{ error.message }}</p>
      </div>
      <Button variant="secondary" @click="refetch()">重试</Button>
    </div>

    <!-- Empty State -->
    <div v-else-if="!sortedProjects.length" class="text-center py-20">
      <div class="text-text-muted mb-6">
        <svg class="w-20 h-20 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
        <h3 class="text-lg font-medium text-text-primary mb-2">还没有项目</h3>
        <p class="text-sm">创建您的第一个项目来开始管理任务</p>
      </div>
      <Button variant="primary" @click="handleCreateProject">
        <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        创建项目
      </Button>
    </div>

    <!-- Project Grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <Card
        v-for="project in sortedProjects"
        :key="project.id"
        class="cursor-pointer hover:shadow-lg hover:border-brand/50 transition-all duration-200"
        @click="handleProjectClick(project.id)"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex-1 min-w-0">
            <h3 class="text-lg font-semibold text-text-primary truncate">
              {{ project.name }}
            </h3>
          </div>
          <div class="ml-3 flex-shrink-0">
            <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <!-- Project Info -->
        <div class="flex items-center gap-1.5 text-sm text-text-muted">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
          </svg>
          <span>点击查看任务看板</span>
        </div>

        <!-- Footer -->
        <div class="mt-4 pt-3 border-t border-border-normal text-xs text-text-muted">
          <span>更新于 {{ formatDate(project.updated_at) }}</span>
        </div>
      </Card>
    </div>
  </div>
</template>
