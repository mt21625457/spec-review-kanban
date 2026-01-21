<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  Settings,
  FolderOpen,
  GitBranch,
  Cpu,
  Server,
  ChevronLeft
} from 'lucide-vue-next'
import { useSettingsDraft } from '@/composables/useSettingsDraft'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { hasUnsavedChanges } = useSettingsDraft()

// 导航项配置
const navigation = computed(() => [
  {
    path: 'general',
    icon: Settings,
    title: t('settings.nav.general'),
    description: t('settings.nav.generalDesc'),
  },
  {
    path: 'projects',
    icon: FolderOpen,
    title: t('settings.nav.projects'),
    description: t('settings.nav.projectsDesc'),
  },
  {
    path: 'repos',
    icon: GitBranch,
    title: t('settings.nav.repos'),
    description: t('settings.nav.reposDesc'),
  },
  {
    path: 'agents',
    icon: Cpu,
    title: t('settings.nav.agents'),
    description: t('settings.nav.agentsDesc'),
  },
  {
    path: 'mcp',
    icon: Server,
    title: t('settings.nav.mcp'),
    description: t('settings.nav.mcpDesc'),
  },
])

// 当前激活的导航项
const currentPath = computed(() => {
  const path = route.path.split('/').pop()
  return path || 'general'
})

// ESC 键关闭设置页面
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    handleClose()
  }
}

// 关闭设置页面
const handleClose = () => {
  if (hasUnsavedChanges.value) {
    const confirmed = window.confirm(t('settings.confirmLeave'))
    if (!confirmed) return
  }
  router.push('/dashboard')
}

// 导航到设置子页面
const navigateTo = (path: string) => {
  if (hasUnsavedChanges.value) {
    const confirmed = window.confirm(t('settings.confirmSwitch'))
    if (!confirmed) return
  }
  router.push(`/settings/${path}`)
}

// 离开页面警告
const handleBeforeUnload = (e: BeforeUnloadEvent) => {
  if (hasUnsavedChanges.value) {
    e.preventDefault()
    e.returnValue = ''
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('beforeunload', handleBeforeUnload)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('beforeunload', handleBeforeUnload)
})
</script>

<template>
  <div class="min-h-screen bg-bg-secondary">
    <!-- 顶部标题栏 -->
    <div class="bg-bg-primary border-b border-border-normal sticky top-0 z-20">
      <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-14">
          <div class="flex items-center gap-3">
            <button
              @click="handleClose"
              class="p-1.5 -ml-1.5 rounded-lg hover:bg-bg-hover transition-colors"
            >
              <ChevronLeft class="w-5 h-5 text-text-normal" />
            </button>
            <h1 class="text-lg font-semibold text-text-primary">
              {{ t('settings.title') }}
            </h1>
          </div>
          <div class="text-xs text-text-low">
            {{ t('settings.escToClose') }}
          </div>
        </div>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <div class="flex flex-col lg:flex-row gap-6">
        <!-- 左侧导航 -->
        <nav class="lg:w-64 flex-shrink-0">
          <div class="lg:sticky lg:top-24 space-y-1">
            <button
              v-for="item in navigation"
              :key="item.path"
              @click="navigateTo(item.path)"
              class="w-full flex items-start gap-3 px-3 py-2.5 rounded-lg text-left transition-colors"
              :class="[
                currentPath === item.path
                  ? 'bg-brand/10 text-brand'
                  : 'hover:bg-bg-hover text-text-normal'
              ]"
            >
              <component
                :is="item.icon"
                class="w-5 h-5 flex-shrink-0 mt-0.5"
                :class="currentPath === item.path ? 'text-brand' : 'text-text-low'"
              />
              <div class="min-w-0">
                <div class="font-medium text-sm">{{ item.title }}</div>
                <div class="text-xs text-text-low mt-0.5 line-clamp-2">
                  {{ item.description }}
                </div>
              </div>
            </button>
          </div>
        </nav>

        <!-- 右侧内容区 -->
        <main class="flex-1 min-w-0">
          <router-view />
        </main>
      </div>
    </div>
  </div>
</template>
