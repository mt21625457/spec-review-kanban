<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useLogout } from '@/composables/useAuth'

const { t } = useI18n()
const authStore = useAuthStore()
const logoutMutation = useLogout()

const isOpen = ref(false)

const isAuthenticated = computed(() => authStore.isAuthenticated)
const isAdmin = computed(() => authStore.user?.role === 'admin')
const displayName = computed(() => authStore.displayName)
const userRole = computed(() => authStore.user?.role)
const userInitial = computed(() => {
  const name = authStore.user?.display_name || authStore.user?.username || ''
  return name.charAt(0).toUpperCase() || '?'
})

const menuItems = [
  { path: '/repo-mappings', label: 'user.repoMappings', icon: 'link' },
  { path: '/settings', label: 'user.settings', icon: 'settings' },
]

const adminMenuItems = [
  { path: '/admin/users', label: 'admin.nav.users', icon: 'users' },
  { path: '/admin/instances', label: 'admin.nav.instances', icon: 'server' },
]

const handleLogout = async () => {
  isOpen.value = false
  await logoutMutation.mutateAsync()
}

const closeMenu = () => {
  isOpen.value = false
}
</script>

<template>
  <div class="relative">
    <!-- 用户头像按钮 -->
    <button
      v-if="isAuthenticated"
      class="flex items-center gap-2 p-1.5 rounded-lg text-text-normal hover:bg-bg-secondary transition-colors"
      @click="isOpen = !isOpen"
    >
      <!-- 头像 -->
      <div class="w-7 h-7 rounded-full bg-brand/20 flex items-center justify-center text-brand font-medium text-sm">
        {{ userInitial }}
      </div>
    </button>

    <!-- 未登录时显示登录按钮 -->
    <RouterLink
      v-else
      to="/login"
      class="px-3 py-1.5 rounded-md bg-brand text-white text-sm font-medium hover:bg-brand/90 transition-colors"
    >
      {{ t('auth.login.title') }}
    </RouterLink>

    <!-- 下拉菜单 -->
    <Transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <div
        v-if="isOpen && isAuthenticated"
        class="absolute right-0 mt-2 w-56 bg-bg-primary border border-border-normal rounded-lg shadow-lg py-1 z-50"
      >
        <!-- 用户信息 -->
        <div class="px-4 py-3 border-b border-border-normal">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-brand/20 flex items-center justify-center text-brand font-medium">
              {{ userInitial }}
            </div>
            <div class="flex-1 min-w-0">
              <div class="font-medium text-text-primary truncate">{{ displayName }}</div>
              <div class="text-xs text-text-muted">
                {{ userRole === 'admin' ? '管理员' : '普通用户' }}
              </div>
            </div>
          </div>
        </div>

        <!-- 菜单项 -->
        <div class="py-1">
          <RouterLink
            v-for="item in menuItems"
            :key="item.path"
            :to="item.path"
            class="flex items-center gap-3 px-4 py-2 text-sm text-text-primary hover:bg-bg-hover transition-colors"
            @click="closeMenu"
          >
            <!-- 图标 -->
            <svg v-if="item.icon === 'link'" class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
            <svg v-else-if="item.icon === 'settings'" class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <span>{{ t(item.label) }}</span>
          </RouterLink>
        </div>

        <!-- 管理员菜单 -->
        <template v-if="isAdmin">
          <div class="border-t border-border-normal my-1" />
          <div class="px-4 py-1.5">
            <span class="text-xs text-text-muted uppercase tracking-wide">{{ t('admin.nav.title') }}</span>
          </div>
          <div class="py-1">
            <RouterLink
              v-for="item in adminMenuItems"
              :key="item.path"
              :to="item.path"
              class="flex items-center gap-3 px-4 py-2 text-sm text-text-primary hover:bg-bg-hover transition-colors"
              @click="closeMenu"
            >
              <!-- 图标 -->
              <svg v-if="item.icon === 'users'" class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
              <svg v-else-if="item.icon === 'server'" class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
              </svg>
              <span>{{ t(item.label) }}</span>
            </RouterLink>
          </div>
        </template>

        <!-- 分隔线 -->
        <div class="border-t border-border-normal my-1" />

        <!-- 登出按钮 -->
        <button
          class="w-full flex items-center gap-3 px-4 py-2 text-sm text-error hover:bg-error/10 transition-colors"
          :disabled="logoutMutation.isPending.value"
          @click="handleLogout"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
          </svg>
          <span>{{ t('user.logout') }}</span>
        </button>
      </div>
    </Transition>
  </div>

  <!-- 点击外部关闭 -->
  <div
    v-if="isOpen"
    class="fixed inset-0 z-40"
    @click="closeMenu"
  />
</template>
