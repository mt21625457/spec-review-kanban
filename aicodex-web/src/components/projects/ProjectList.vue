<script setup lang="ts">
/**
 * ProjectList - Display a list of projects with management actions
 */
import { computed } from 'vue'
import { useProjects } from '@/composables/useProjects'
import { useProjectsStore, useUiStore } from '@/stores'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import Loading from '@/components/ui/Loading.vue'
import type { Project } from '@/types'
import { formatDistanceToNow } from 'date-fns'
import { zhCN } from 'date-fns/locale'

const { data: projects, isLoading, error, refetch } = useProjects()
const projectsStore = useProjectsStore()
const uiStore = useUiStore()

const sortedProjects = computed(() => {
  if (!projects.value) return []
  return [...projects.value].sort((a, b) =>
    new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
  )
})

const formatDate = (dateStr: string) => {
  try {
    return formatDistanceToNow(new Date(dateStr), {
      addSuffix: true,
      locale: zhCN,
    })
  } catch {
    return dateStr
  }
}

const handleSelect = (project: Project) => {
  projectsStore.setCurrentProject(project.id)
}

const handleEdit = (project: Project) => {
  uiStore.openDialog('editProject', project)
}

const handleDelete = (project: Project) => {
  uiStore.openDialog('deleteProject', project)
}

const handleCreate = () => {
  uiStore.openDialog('createProject')
}
</script>

<template>
  <div class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold text-text-primary">项目列表</h2>
      <Button variant="primary" size="sm" @click="handleCreate">
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        新建项目
      </Button>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex justify-center py-12">
      <Loading />
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="text-center py-12">
      <div class="text-red-500 mb-4">
        <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="text-text-secondary">加载项目列表失败</p>
      </div>
      <Button variant="secondary" @click="refetch">重试</Button>
    </div>

    <!-- Empty State -->
    <div v-else-if="!sortedProjects.length" class="text-center py-12">
      <div class="text-text-muted mb-4">
        <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <p>暂无项目</p>
      </div>
      <Button variant="primary" @click="handleCreate">创建第一个项目</Button>
    </div>

    <!-- Project Grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <Card
        v-for="project in sortedProjects"
        :key="project.id"
        class="group cursor-pointer hover:border-brand/50 transition-colors"
        :class="{
          'ring-2 ring-brand border-brand': project.id === projectsStore.currentProjectId
        }"
        @click="handleSelect(project)"
      >
        <div class="p-4">
          <!-- Header -->
          <div class="flex items-start justify-between mb-2">
            <div class="flex items-center gap-2 min-w-0">
              <svg class="w-5 h-5 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <h3 class="font-medium text-text-primary truncate">{{ project.name }}</h3>
            </div>

            <!-- Actions Menu -->
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                type="button"
                class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
                title="编辑"
                @click.stop="handleEdit(project)"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
              </button>
              <button
                type="button"
                class="p-1 text-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                title="删除"
                @click.stop="handleDelete(project)"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>

          <!-- Meta Info -->
          <div class="text-sm text-text-muted space-y-1">
            <div class="flex items-center gap-1">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span>{{ formatDate(project.updated_at) }}</span>
            </div>

            <!-- Current Project Badge -->
            <div
              v-if="project.id === projectsStore.currentProjectId"
              class="inline-flex items-center gap-1 px-2 py-0.5 bg-brand/10 text-brand text-xs rounded-full"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              当前项目
            </div>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
