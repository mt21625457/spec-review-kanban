import { useThemeStore } from '@/stores/theme'
import { storeToRefs } from 'pinia'

export function useTheme() {
  const store = useThemeStore()
  const { mode, isDark } = storeToRefs(store)

  return {
    mode,
    isDark,
    setTheme: store.setTheme,
  }
}
