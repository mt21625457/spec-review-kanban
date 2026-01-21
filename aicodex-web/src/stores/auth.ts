/**
 * Auth Store - 用户认证状态管理
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserInfo, InstanceInfo } from '@/types'
import { getAuthToken, setAuthToken } from '@/lib/api'

export const useAuthStore = defineStore('auth', () => {
  // 状态
  const user = ref<UserInfo | null>(null)
  const instances = ref<InstanceInfo[]>([])
  const currentInstanceId = ref<string | null>(null)
  const isInitialized = ref(false)
  const isLoading = ref(false)

  // Getters
  const isAuthenticated = computed(() => !!user.value && !!getAuthToken())
  const isAdmin = computed(() => user.value?.role === 'admin')
  const currentInstance = computed(() =>
    instances.value.find((i) => i.id === currentInstanceId.value)
  )
  const displayName = computed(
    () => user.value?.display_name || user.value?.username || '用户'
  )

  // Actions
  function setUser(userData: UserInfo | null) {
    user.value = userData
    if (userData) {
      currentInstanceId.value = userData.current_instance_id
    } else {
      currentInstanceId.value = null
    }
  }

  function setInstances(instanceList: InstanceInfo[]) {
    instances.value = instanceList
  }

  function setCurrentInstanceId(id: string | null) {
    currentInstanceId.value = id
    if (user.value) {
      user.value.current_instance_id = id
    }
  }

  function updateInstance(updatedInstance: InstanceInfo) {
    const index = instances.value.findIndex((i) => i.id === updatedInstance.id)
    if (index !== -1) {
      instances.value[index] = updatedInstance
    }
  }

  function clear() {
    user.value = null
    instances.value = []
    currentInstanceId.value = null
    setAuthToken(null)
  }

  function setInitialized(value: boolean) {
    isInitialized.value = value
  }

  function setLoading(value: boolean) {
    isLoading.value = value
  }

  // 从 localStorage 初始化
  function init() {
    const token = getAuthToken()
    if (!token) {
      isInitialized.value = true
    }
    // 实际的用户信息需要通过 API 获取
  }

  return {
    // State
    user,
    instances,
    currentInstanceId,
    isInitialized,
    isLoading,
    // Getters
    isAuthenticated,
    isAdmin,
    currentInstance,
    displayName,
    // Actions
    setUser,
    setInstances,
    setCurrentInstanceId,
    updateInstance,
    clear,
    setInitialized,
    setLoading,
    init,
  }
})
