import { ref, computed } from 'vue'
import { cloneDeep, isEqual } from 'lodash-es'

/**
 * 设置草稿管理 composable
 * 用于管理设置页面的草稿状态、脏标志和保存操作
 */

// 全局草稿状态
const draft = ref<Record<string, unknown> | null>(null)
const original = ref<Record<string, unknown> | null>(null)
const saving = ref(false)
const success = ref(false)
const error = ref<string | null>(null)

export function useSettingsDraft<T extends Record<string, unknown>>() {
  // 计算是否有未保存的更改
  const hasUnsavedChanges = computed(() => {
    if (!draft.value || !original.value) return false
    return !isEqual(draft.value, original.value)
  })

  // 初始化草稿（从服务器加载数据后调用）
  const initDraft = (data: T) => {
    original.value = cloneDeep(data)
    draft.value = cloneDeep(data)
    success.value = false
    error.value = null
  }

  // 更新草稿的部分字段
  const updateDraft = (patch: Partial<T>) => {
    if (!draft.value) return
    draft.value = { ...draft.value, ...patch }
  }

  // 设置草稿的特定字段
  const setDraftField = <K extends keyof T>(key: K, value: T[K]) => {
    if (!draft.value) return
    draft.value = { ...draft.value, [key]: value }
  }

  // 重置草稿到原始状态
  const resetDraft = () => {
    if (!original.value) return
    draft.value = cloneDeep(original.value)
    success.value = false
    error.value = null
  }

  // 保存草稿
  const saveDraft = async (saveFn: (data: T) => Promise<void>) => {
    if (!draft.value) return false

    saving.value = true
    error.value = null
    success.value = false

    try {
      await saveFn(draft.value as T)
      original.value = cloneDeep(draft.value)
      success.value = true
      // 3秒后清除成功状态
      setTimeout(() => {
        success.value = false
      }, 3000)
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : '保存失败'
      return false
    } finally {
      saving.value = false
    }
  }

  // 清除状态
  const clearDraft = () => {
    draft.value = null
    original.value = null
    saving.value = false
    success.value = false
    error.value = null
  }

  return {
    draft: draft as unknown as ReturnType<typeof ref<T | null>>,
    original: original as unknown as ReturnType<typeof ref<T | null>>,
    hasUnsavedChanges,
    saving,
    success,
    error,
    initDraft,
    updateDraft,
    setDraftField,
    resetDraft,
    saveDraft,
    clearDraft,
  }
}

/**
 * 单独的草稿管理 composable（用于每个设置页面独立管理）
 */
export function useLocalDraft<T extends object>() {
  const localDraft = ref<T | null>(null)
  const localOriginal = ref<T | null>(null)
  const localSaving = ref(false)
  const localSuccess = ref(false)
  const localError = ref<string | null>(null)

  const hasChanges = computed(() => {
    if (!localDraft.value || !localOriginal.value) return false
    return !isEqual(localDraft.value, localOriginal.value)
  })

  const init = (data: T) => {
    localOriginal.value = cloneDeep(data) as T
    localDraft.value = cloneDeep(data) as T
    localSuccess.value = false
    localError.value = null
  }

  const update = (patch: Partial<T>) => {
    if (!localDraft.value) return
    localDraft.value = { ...localDraft.value, ...patch } as T
  }

  const reset = () => {
    if (!localOriginal.value) return
    localDraft.value = cloneDeep(localOriginal.value) as T
    localSuccess.value = false
    localError.value = null
  }

  const save = async (saveFn: (data: T) => Promise<void>) => {
    if (!localDraft.value) return false

    localSaving.value = true
    localError.value = null
    localSuccess.value = false

    try {
      await saveFn(localDraft.value)
      localOriginal.value = cloneDeep(localDraft.value) as T
      localSuccess.value = true
      setTimeout(() => {
        localSuccess.value = false
      }, 3000)
      return true
    } catch (err) {
      localError.value = err instanceof Error ? err.message : '保存失败'
      return false
    } finally {
      localSaving.value = false
    }
  }

  const clear = () => {
    localDraft.value = null
    localOriginal.value = null
    localSaving.value = false
    localSuccess.value = false
    localError.value = null
  }

  return {
    draft: localDraft,
    original: localOriginal,
    hasChanges,
    saving: localSaving,
    success: localSuccess,
    error: localError,
    init,
    update,
    reset,
    save,
    clear,
  }
}
