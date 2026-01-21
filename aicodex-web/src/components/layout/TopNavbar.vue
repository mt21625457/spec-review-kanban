<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ThemeToggle from './ThemeToggle.vue'
import LanguageSwitcher from './LanguageSwitcher.vue'
import UserMenu from './UserMenu.vue'
import InstanceSelector from './InstanceSelector.vue'
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

const { t } = useI18n()
const route = useRoute()
const isMobileMenuOpen = ref(false)
const authStore = useAuthStore()

const navLinks = [
  { path: '/dashboard', label: 'nav.dashboard' },
  { path: '/reviews', label: 'nav.reviews' },
  { path: '/tasks', label: 'nav.tasks' },
]

const isActive = (path: string) => {
  if (path === '/dashboard') {
    return route.path === '/' || route.path === '/dashboard'
  }
  return route.path.startsWith(path)
}

const toggleMobileMenu = () => {
  isMobileMenuOpen.value = !isMobileMenuOpen.value
}
</script>

<template>
  <nav class="fixed top-0 left-0 right-0 z-50 h-navbar bg-bg-primary border-b border-border">
    <div class="h-full max-w-7xl mx-auto px-4 flex items-center justify-between">
      <!-- 左侧: Logo + 导航 -->
      <div class="flex items-center gap-8">
        <!-- Logo -->
        <RouterLink to="/" class="flex items-center gap-2 text-text-high font-semibold">
          <svg class="w-8 h-8 text-brand" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
          </svg>
          <span class="hidden sm:inline">多智能体编排系统</span>
        </RouterLink>

        <!-- 桌面端导航链接 -->
        <div class="hidden md:flex items-center gap-1">
          <RouterLink
            v-for="link in navLinks"
            :key="link.path"
            :to="link.path"
            class="px-3 py-2 rounded-md text-sm font-medium transition-colors"
            :class="isActive(link.path)
              ? 'bg-brand/10 text-brand'
              : 'text-text-normal hover:text-text-high hover:bg-bg-secondary'"
          >
            {{ t(link.label) }}
          </RouterLink>
        </div>
      </div>

      <!-- 右侧: 工具按钮 -->
      <div class="flex items-center gap-2">
        <!-- 实例选择器 -->
        <InstanceSelector v-if="authStore.isAuthenticated" class="hidden sm:block" />

        <!-- 分隔线 -->
        <div v-if="authStore.isAuthenticated" class="hidden sm:block w-px h-6 bg-border-normal" />

        <!-- 设置按钮 -->
        <RouterLink
          to="/settings"
          class="p-2 rounded-md transition-colors"
          :class="isActive('/settings')
            ? 'bg-brand/10 text-brand'
            : 'text-text-normal hover:text-text-high hover:bg-bg-secondary'"
          :title="t('nav.settings')"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </RouterLink>

        <ThemeToggle />
        <LanguageSwitcher />
        <UserMenu />

        <!-- 移动端菜单按钮 -->
        <button
          class="md:hidden p-2 rounded-md text-text-normal hover:bg-bg-secondary"
          @click="toggleMobileMenu"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              v-if="!isMobileMenuOpen"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 12h16M4 18h16"
            />
            <path
              v-else
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- 移动端导航菜单 -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-1"
    >
      <div
        v-if="isMobileMenuOpen"
        class="md:hidden border-t border-border bg-bg-primary shadow-lg"
      >
        <div class="px-4 py-2 space-y-1">
          <RouterLink
            v-for="link in navLinks"
            :key="link.path"
            :to="link.path"
            class="block px-3 py-2 rounded-md text-sm font-medium transition-colors"
            :class="isActive(link.path)
              ? 'bg-brand/10 text-brand'
              : 'text-text-normal hover:text-text-high hover:bg-bg-secondary'"
            @click="isMobileMenuOpen = false"
          >
            {{ t(link.label) }}
          </RouterLink>
        </div>
      </div>
    </Transition>
  </nav>
</template>
