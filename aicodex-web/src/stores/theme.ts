import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>((localStorage.getItem('theme') as ThemeMode) || 'system')
  const isDark = ref(false)

  const updateTheme = () => {
    if (mode.value === 'system') {
      isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
    } else {
      isDark.value = mode.value === 'dark'
    }
    document.documentElement.classList.toggle('dark', isDark.value)
  }

  const setTheme = (newMode: ThemeMode) => {
    mode.value = newMode
    localStorage.setItem('theme', newMode)
    updateTheme()
  }

  // 初始化
  updateTheme()

  // 监听系统主题变化
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (mode.value === 'system') {
      updateTheme()
    }
  })

  return {
    mode,
    isDark,
    setTheme,
  }
})
