/**
 * useAuth - 用户认证 composable
 * 提供登录、登出、获取当前用户等功能
 */
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { authApi, myInstancesApi, getAuthToken } from '@/lib/api'
import { queryKeys } from '@/lib/queryClient'
import { useAuthStore } from '@/stores/auth'
import type { LoginRequest, RegisterRequest, ChangePasswordRequest, InstanceInfo } from '@/types'

/**
 * 获取当前用户信息
 */
export function useCurrentUser() {
  const authStore = useAuthStore()
  const token = getAuthToken()

  const query = useQuery({
    queryKey: queryKeys.auth.me,
    queryFn: async () => {
      const response = await authApi.me()
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '获取用户信息失败')
    },
    enabled: !!token,
    staleTime: 5 * 60 * 1000, // 5 分钟
    retry: false, // 认证失败不重试
  })

  // 同步到 store
  if (query.data.value) {
    authStore.setUser(query.data.value.user)
    authStore.setInstances(query.data.value.instances)
    authStore.setInitialized(true)
  }

  return query
}

/**
 * 用户登录
 */
export function useLogin() {
  const queryClient = useQueryClient()
  const authStore = useAuthStore()

  return useMutation({
    mutationFn: async (data: LoginRequest) => {
      const response = await authApi.login(data)
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '登录失败')
    },
    onSuccess: (data) => {
      authStore.setUser(data.user)
      authStore.setInstances(data.instances)
      authStore.setCurrentInstanceId(data.current_instance_id)
      authStore.setInitialized(true)
      // 刷新用户信息缓存
      queryClient.setQueryData(queryKeys.auth.me, {
        user: data.user,
        instances: data.instances,
      })
    },
  })
}

/**
 * 用户注册
 */
export function useRegister() {
  return useMutation({
    mutationFn: async (data: RegisterRequest) => {
      const response = await authApi.register(data)
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '注册失败')
    },
  })
}

/**
 * 用户登出
 */
export function useLogout() {
  const queryClient = useQueryClient()
  const authStore = useAuthStore()
  const router = useRouter()

  return useMutation({
    mutationFn: async () => {
      await authApi.logout()
    },
    onSuccess: () => {
      authStore.clear()
      queryClient.clear()
      router.push('/login')
    },
    onError: () => {
      // 即使 API 失败，也清理本地状态
      authStore.clear()
      queryClient.clear()
      router.push('/login')
    },
  })
}

/**
 * 修改密码
 */
export function useChangePassword() {
  return useMutation({
    mutationFn: async (data: ChangePasswordRequest) => {
      const response = await authApi.changePassword(data)
      if (response.success) {
        return response.data
      }
      throw new Error(response.error || '修改密码失败')
    },
  })
}

/**
 * 获取用户实例列表
 */
export function useMyInstances() {
  const authStore = useAuthStore()

  return useQuery({
    queryKey: queryKeys.myInstances.all,
    queryFn: async () => {
      const response = await myInstancesApi.list()
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '获取实例列表失败')
    },
    enabled: computed(() => authStore.isAuthenticated),
    staleTime: 30 * 1000, // 30 秒
  })
}

/**
 * 获取当前实例
 */
export function useCurrentInstance() {
  const authStore = useAuthStore()

  return useQuery({
    queryKey: queryKeys.myInstances.current,
    queryFn: async () => {
      const response = await myInstancesApi.getCurrent()
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '获取当前实例失败')
    },
    enabled: computed(() => authStore.isAuthenticated && !!authStore.currentInstanceId),
    staleTime: 30 * 1000,
  })
}

/**
 * 切换实例
 */
export function useSwitchInstance() {
  const queryClient = useQueryClient()
  const authStore = useAuthStore()

  return useMutation({
    mutationFn: async (instanceId: string) => {
      const response = await myInstancesApi.switchInstance({ instance_id: instanceId })
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '切换实例失败')
    },
    onSuccess: (instance: InstanceInfo) => {
      authStore.setCurrentInstanceId(instance.id)
      authStore.updateInstance(instance)
      // 刷新当前实例缓存
      queryClient.setQueryData(queryKeys.myInstances.current, instance)
      // 刷新实例列表
      queryClient.invalidateQueries({ queryKey: queryKeys.myInstances.all })
      // 刷新所有代理相关的数据
      queryClient.invalidateQueries({ queryKey: queryKeys.projects.all })
      queryClient.invalidateQueries({ queryKey: queryKeys.tasks.all })
    },
  })
}

/**
 * 当前实例健康检查
 */
export function useInstanceHealth() {
  const authStore = useAuthStore()

  return useQuery({
    queryKey: queryKeys.myInstances.health,
    queryFn: async () => {
      const response = await myInstancesApi.currentHealth()
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '健康检查失败')
    },
    enabled: computed(() => authStore.isAuthenticated && !!authStore.currentInstanceId),
    staleTime: 10 * 1000, // 10 秒
    refetchInterval: 30 * 1000, // 每 30 秒刷新
  })
}

/**
 * 认证初始化 hook
 * 在应用启动时调用，用于恢复登录状态
 */
export function useAuthInit() {
  const authStore = useAuthStore()
  const currentUserQuery = useCurrentUser()

  onMounted(async () => {
    const token = getAuthToken()
    if (token) {
      authStore.setLoading(true)
      try {
        await currentUserQuery.refetch()
      } finally {
        authStore.setLoading(false)
        authStore.setInitialized(true)
      }
    } else {
      authStore.setInitialized(true)
    }
  })

  return {
    isLoading: computed(() => authStore.isLoading || currentUserQuery.isLoading.value),
    isInitialized: computed(() => authStore.isInitialized),
    isAuthenticated: computed(() => authStore.isAuthenticated),
    user: computed(() => authStore.user),
    error: currentUserQuery.error,
  }
}
