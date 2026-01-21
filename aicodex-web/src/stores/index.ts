import { createPinia } from 'pinia'

const pinia = createPinia()

export default pinia

// Store exports
export { useTasksStore, type Task as LegacyTask, type Project as LegacyProject } from './tasks'
export { useThemeStore, type ThemeMode } from './theme'
export { useProjectsStore } from './projects'
export { useUiStore, type PanelType, type DialogType } from './ui'
export { useAuthStore } from './auth'
