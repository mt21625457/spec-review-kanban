/**
 * UI Store - Client state for UI elements (panels, dialogs, etc.)
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

// Panel types
export type PanelType = 'task' | 'attempt' | 'diffs' | 'preview' | 'settings' | null

// Dialog types
export type DialogType =
  | 'createProject'
  | 'editProject'
  | 'deleteProject'
  | 'createTask'
  | 'editTask'
  | 'deleteTask'
  | 'linkSharedTask'
  | 'shareTask'
  | 'gitPush'
  | 'createPR'
  | 'keyboardShortcuts'
  | null

export const useUiStore = defineStore('ui', () => {
  // Panel state
  const activePanelType = ref<PanelType>(null)
  const selectedTaskId = ref<string | null>(null)
  const selectedAttemptId = ref<string | null>(null)

  // Dialog state
  const activeDialog = ref<DialogType>(null)
  const dialogData = ref<unknown>(null)

  // Sidebar state
  const sidebarCollapsed = ref(false)

  // Search state
  const searchQuery = ref('')
  const isSearchOpen = ref(false)

  // Task filters
  const statusFilter = ref<string[]>([])
  const priorityFilter = ref<string[]>([])
  const tagFilter = ref<string[]>([])

  // Selected tasks for batch operations
  const selectedTaskIds = ref<Set<string>>(new Set())

  // Loading states
  const isLoading = ref(false)
  const loadingMessage = ref('')

  // Getters
  const isPanelOpen = computed(() => activePanelType.value !== null)
  const isDialogOpen = computed(() => activeDialog.value !== null)
  const hasSelectedTasks = computed(() => selectedTaskIds.value.size > 0)
  const selectedTaskCount = computed(() => selectedTaskIds.value.size)

  // Panel actions
  function openTaskPanel(taskId: string) {
    selectedTaskId.value = taskId
    selectedAttemptId.value = null
    activePanelType.value = 'task'
  }

  function openAttemptPanel(attemptId: string) {
    selectedAttemptId.value = attemptId
    activePanelType.value = 'attempt'
  }

  function openDiffsPanel(attemptId: string) {
    selectedAttemptId.value = attemptId
    activePanelType.value = 'diffs'
  }

  function openPreviewPanel(attemptId: string) {
    selectedAttemptId.value = attemptId
    activePanelType.value = 'preview'
  }

  function openSettingsPanel() {
    activePanelType.value = 'settings'
  }

  function closePanel() {
    activePanelType.value = null
    selectedTaskId.value = null
    selectedAttemptId.value = null
  }

  // Dialog actions
  function openDialog(type: DialogType, data?: unknown) {
    activeDialog.value = type
    dialogData.value = data ?? null
  }

  function closeDialog() {
    activeDialog.value = null
    dialogData.value = null
  }

  // Sidebar actions
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function setSidebarCollapsed(collapsed: boolean) {
    sidebarCollapsed.value = collapsed
  }

  // Search actions
  function openSearch() {
    isSearchOpen.value = true
  }

  function closeSearch() {
    isSearchOpen.value = false
    searchQuery.value = ''
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  // Filter actions
  function setStatusFilter(statuses: string[]) {
    statusFilter.value = statuses
  }

  function setPriorityFilter(priorities: string[]) {
    priorityFilter.value = priorities
  }

  function setTagFilter(tags: string[]) {
    tagFilter.value = tags
  }

  function clearFilters() {
    statusFilter.value = []
    priorityFilter.value = []
    tagFilter.value = []
    searchQuery.value = ''
  }

  // Task selection actions
  function selectTask(taskId: string) {
    selectedTaskIds.value.add(taskId)
  }

  function deselectTask(taskId: string) {
    selectedTaskIds.value.delete(taskId)
  }

  function toggleTaskSelection(taskId: string) {
    if (selectedTaskIds.value.has(taskId)) {
      selectedTaskIds.value.delete(taskId)
    } else {
      selectedTaskIds.value.add(taskId)
    }
  }

  function selectAllTasks(taskIds: string[]) {
    selectedTaskIds.value = new Set(taskIds)
  }

  function clearTaskSelection() {
    selectedTaskIds.value.clear()
  }

  // Loading actions
  function setLoading(loading: boolean, message = '') {
    isLoading.value = loading
    loadingMessage.value = message
  }

  // Keyboard shortcut handler
  function handleKeyboardShortcut(event: KeyboardEvent) {
    // Escape - close panel/dialog/search
    if (event.key === 'Escape') {
      if (isDialogOpen.value) {
        closeDialog()
      } else if (isSearchOpen.value) {
        closeSearch()
      } else if (isPanelOpen.value) {
        closePanel()
      } else if (hasSelectedTasks.value) {
        clearTaskSelection()
      }
      return true
    }

    // Cmd/Ctrl + K - open search
    if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
      event.preventDefault()
      openSearch()
      return true
    }

    // ? - show keyboard shortcuts
    if (event.key === '?' && !event.metaKey && !event.ctrlKey) {
      openDialog('keyboardShortcuts')
      return true
    }

    return false
  }

  return {
    // Panel state
    activePanelType,
    selectedTaskId,
    selectedAttemptId,
    isPanelOpen,
    // Dialog state
    activeDialog,
    dialogData,
    isDialogOpen,
    // Sidebar state
    sidebarCollapsed,
    // Search state
    searchQuery,
    isSearchOpen,
    // Filter state
    statusFilter,
    priorityFilter,
    tagFilter,
    // Selection state
    selectedTaskIds,
    hasSelectedTasks,
    selectedTaskCount,
    // Loading state
    isLoading,
    loadingMessage,
    // Panel actions
    openTaskPanel,
    openAttemptPanel,
    openDiffsPanel,
    openPreviewPanel,
    openSettingsPanel,
    closePanel,
    // Dialog actions
    openDialog,
    closeDialog,
    // Sidebar actions
    toggleSidebar,
    setSidebarCollapsed,
    // Search actions
    openSearch,
    closeSearch,
    setSearchQuery,
    // Filter actions
    setStatusFilter,
    setPriorityFilter,
    setTagFilter,
    clearFilters,
    // Selection actions
    selectTask,
    deselectTask,
    toggleTaskSelection,
    selectAllTasks,
    clearTaskSelection,
    // Loading actions
    setLoading,
    // Keyboard shortcuts
    handleKeyboardShortcut,
  }
})
