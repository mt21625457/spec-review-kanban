<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useSwitchInstance, useMyInstances } from '@/composables/useAuth'
import { Badge } from '@/components/ui'
import type { InstanceInfo } from '@/types'

const { t } = useI18n()
const authStore = useAuthStore()
const switchInstanceMutation = useSwitchInstance()
const { data: instancesData, isLoading } = useMyInstances()

const isOpen = ref(false)
const dropdownRef = ref<HTMLDivElement | null>(null)

const instances = computed(() => instancesData.value?.instances || authStore.instances)
const currentInstance = computed(() => authStore.currentInstance)
const hasMultipleInstances = computed(() => instances.value.length > 1)

const getStatusText = (status: string) => {
  return t(`instance.selector.status.${status}`)
}

const toggleDropdown = () => {
  if (hasMultipleInstances.value) {
    isOpen.value = !isOpen.value
  }
}

const selectInstance = async (instance: InstanceInfo) => {
  if (instance.id === authStore.currentInstanceId) {
    isOpen.value = false
    return
  }

  try {
    await switchInstanceMutation.mutateAsync(instance.id)
    isOpen.value = false
  } catch (error) {
    console.error('切换实例失败:', error)
  }
}

// 点击外部关闭
const handleClickOutside = (event: MouseEvent) => {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

// 监听点击事件
if (typeof window !== 'undefined') {
  document.addEventListener('click', handleClickOutside)
}
</script>

<template>
  <div ref="dropdownRef" class="relative">
    <!-- 当前实例按钮 -->
    <button
      v-if="currentInstance"
      class="flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors"
      :class="[
        hasMultipleInstances
          ? 'hover:bg-bg-hover cursor-pointer'
          : 'cursor-default',
        isOpen ? 'bg-bg-hover' : ''
      ]"
      @click="toggleDropdown"
    >
      <!-- 状态指示器 -->
      <span
        class="w-2 h-2 rounded-full"
        :class="{
          'bg-success': currentInstance.status === 'running',
          'bg-text-muted': currentInstance.status === 'stopped',
          'bg-warning animate-pulse': currentInstance.status === 'starting' || currentInstance.status === 'stopping',
          'bg-error': currentInstance.status === 'error',
        }"
      />

      <!-- 实例名称 -->
      <span class="text-sm font-medium text-text-primary truncate max-w-[120px]">
        {{ currentInstance.name }}
      </span>

      <!-- 下拉箭头 -->
      <svg
        v-if="hasMultipleInstances"
        class="w-4 h-4 text-text-muted transition-transform"
        :class="{ 'rotate-180': isOpen }"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- 无实例提示 -->
    <div v-else-if="!isLoading" class="text-sm text-text-muted px-3 py-1.5">
      {{ t('instance.selector.noInstances') }}
    </div>

    <!-- 加载中 -->
    <div v-else class="text-sm text-text-muted px-3 py-1.5">
      {{ t('common.loading') }}
    </div>

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
        v-if="isOpen && hasMultipleInstances"
        class="absolute right-0 mt-2 w-64 rounded-lg bg-bg-primary shadow-lg ring-1 ring-border-normal z-50"
      >
        <div class="p-2">
          <div class="px-2 py-1.5 text-xs font-medium text-text-muted uppercase tracking-wide">
            {{ t('instance.selector.title') }}
          </div>

          <div class="mt-1 space-y-1">
            <button
              v-for="instance in instances"
              :key="instance.id"
              class="w-full flex items-center gap-3 px-2 py-2 rounded-md transition-colors text-left"
              :class="[
                instance.id === authStore.currentInstanceId
                  ? 'bg-brand/10 text-brand'
                  : 'hover:bg-bg-hover text-text-primary'
              ]"
              :disabled="switchInstanceMutation.isPending.value"
              @click="selectInstance(instance)"
            >
              <!-- 状态指示器 -->
              <span
                class="w-2 h-2 rounded-full flex-shrink-0"
                :class="{
                  'bg-success': instance.status === 'running',
                  'bg-text-muted': instance.status === 'stopped',
                  'bg-warning animate-pulse': instance.status === 'starting' || instance.status === 'stopping',
                  'bg-error': instance.status === 'error',
                }"
              />

              <!-- 实例信息 -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-medium truncate">{{ instance.name }}</span>
                  <Badge
                    v-if="instance.id === authStore.currentInstanceId"
                    variant="brand"
                    class="text-xs"
                  >
                    {{ t('instance.selector.current') }}
                  </Badge>
                </div>
                <div class="text-xs text-text-muted">
                  {{ getStatusText(instance.status) }}
                  <span v-if="instance.user_count !== undefined">
                    · {{ instance.user_count }} 用户
                  </span>
                </div>
              </div>

              <!-- 选中指示 -->
              <svg
                v-if="instance.id === authStore.currentInstanceId"
                class="w-4 h-4 text-brand flex-shrink-0"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
