<script setup lang="ts">
/**
 * ProjectSelector - Dropdown for selecting the current project
 */
import { computed, ref } from 'vue'
import { useProjectsStore } from '@/stores'
import { useProjects } from '@/composables/useProjects'
import { useUiStore } from '@/stores'

const projectsStore = useProjectsStore()
const uiStore = useUiStore()
const { data: projects, isLoading, error } = useProjects()

const isOpen = ref(false)

const currentProject = computed(() =>
  projects.value?.find((p) => p.id === projectsStore.currentProjectId)
)

const sortedProjects = computed(() => {
  if (!projects.value) return []
  return [...projects.value].sort((a, b) => a.name.localeCompare(b.name))
})

const selectProject = (projectId: string) => {
  projectsStore.setCurrentProject(projectId)
  isOpen.value = false
}

const openCreateDialog = () => {
  uiStore.openDialog('createProject')
  isOpen.value = false
}

const closeDropdown = () => {
  isOpen.value = false
}
</script>

<template>
  <div class="relative" v-click-outside="closeDropdown">
    <!-- Trigger Button -->
    <button
      type="button"
      class="flex items-center gap-2 px-3 py-2 text-sm font-medium text-text-primary bg-bg-secondary hover:bg-bg-hover border border-border-normal rounded-lg transition-colors min-w-[180px]"
      @click="isOpen = !isOpen"
    >
      <!-- Project Icon -->
      <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>

      <!-- Loading State -->
      <template v-if="isLoading">
        <span class="flex-1 text-left text-text-muted">加载中...</span>
      </template>

      <!-- Error State -->
      <template v-else-if="error">
        <span class="flex-1 text-left text-red-500">加载失败</span>
      </template>

      <!-- No Projects -->
      <template v-else-if="!projects?.length">
        <span class="flex-1 text-left text-text-muted">无项目</span>
      </template>

      <!-- Current Project -->
      <template v-else>
        <span class="flex-1 text-left truncate">
          {{ currentProject?.name || '选择项目' }}
        </span>
      </template>

      <!-- Chevron -->
      <svg
        class="w-4 h-4 text-text-muted transition-transform"
        :class="{ 'rotate-180': isOpen }"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Dropdown Menu -->
    <Transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <div
        v-if="isOpen"
        class="absolute z-50 mt-1 w-full min-w-[240px] bg-bg-primary border border-border-normal rounded-lg shadow-lg py-1 max-h-80 overflow-auto"
      >
        <!-- Project List -->
        <div v-if="sortedProjects.length" class="py-1">
          <button
            v-for="project in sortedProjects"
            :key="project.id"
            type="button"
            class="w-full px-3 py-2 text-left text-sm hover:bg-bg-hover transition-colors flex items-center gap-2"
            :class="{
              'bg-brand/10 text-brand': project.id === projectsStore.currentProjectId
            }"
            @click="selectProject(project.id)"
          >
            <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <span class="truncate">{{ project.name }}</span>
            <svg
              v-if="project.id === projectsStore.currentProjectId"
              class="w-4 h-4 ml-auto text-brand flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </button>
        </div>

        <!-- Empty State -->
        <div v-else class="px-3 py-6 text-center text-text-muted text-sm">
          暂无项目
        </div>

        <!-- Divider -->
        <div class="border-t border-border-normal my-1"></div>

        <!-- Create New Project -->
        <button
          type="button"
          class="w-full px-3 py-2 text-left text-sm text-brand hover:bg-bg-hover transition-colors flex items-center gap-2"
          @click="openCreateDialog"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          <span>新建项目</span>
        </button>
      </div>
    </Transition>
  </div>
</template>
