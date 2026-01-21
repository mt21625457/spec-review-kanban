<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTheme } from '@/composables/useTheme'

const { t } = useI18n()
const { mode, setTheme } = useTheme()
const isOpen = ref(false)

const themes = [
  { value: 'light', label: 'theme.light', icon: '☀️' },
  { value: 'dark', label: 'theme.dark', icon: '🌙' },
  { value: 'system', label: 'theme.system', icon: '💻' },
] as const

const selectTheme = (value: typeof mode.value) => {
  setTheme(value)
  isOpen.value = false
}

const currentIcon = () => {
  return themes.find(th => th.value === mode.value)?.icon || '🎨'
}
</script>

<template>
  <div class="relative">
    <button
      class="p-2 rounded-md text-text-normal hover:bg-bg-secondary transition-colors"
      @click="isOpen = !isOpen"
      :title="t('theme.' + mode)"
    >
      <span class="text-lg">{{ currentIcon() }}</span>
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 mt-2 w-40 bg-bg-primary border border-border rounded-lg shadow-lg py-1 z-50"
    >
      <button
        v-for="theme in themes"
        :key="theme.value"
        class="w-full px-4 py-2 text-left text-sm flex items-center gap-2 hover:bg-bg-secondary transition-colors"
        :class="mode === theme.value ? 'text-brand' : 'text-text-normal'"
        @click="selectTheme(theme.value)"
      >
        <span>{{ theme.icon }}</span>
        <span>{{ t(theme.label) }}</span>
      </button>
    </div>
  </div>

  <!-- 点击外部关闭 -->
  <div
    v-if="isOpen"
    class="fixed inset-0 z-40"
    @click="isOpen = false"
  />
</template>
