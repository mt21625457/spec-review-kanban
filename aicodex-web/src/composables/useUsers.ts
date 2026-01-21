/**
 * useUsers - 用户管理 composable (管理员)
 */
import { computed, type Ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { usersApi, type CreateUserRequest, type UpdateUserRequest } from '@/lib/api'
import type { UserInfo, InstanceInfo } from '@/types'

// Query keys
export const userQueryKeys = {
  all: ['admin', 'users'] as const,
  detail: (id: string) => ['admin', 'users', id] as const,
  instances: (id: string) => ['admin', 'users', id, 'instances'] as const,
}

/**
 * 获取所有用户列表
 */
export function useUsers() {
  return useQuery({
    queryKey: userQueryKeys.all,
    queryFn: async () => {
      const response = await usersApi.list()
      if (response.success && response.data) {
        return response.data.users
      }
      throw new Error(response.error || '获取用户列表失败')
    },
    staleTime: 30 * 1000,
  })
}

/**
 * 获取用户详情
 */
export function useUser(userId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      userId.value ? userQueryKeys.detail(userId.value) : ['admin', 'users', null]
    ),
    queryFn: async () => {
      if (!userId.value) throw new Error('用户 ID 不能为空')
      const response = await usersApi.get(userId.value)
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '获取用户详情失败')
    },
    enabled: computed(() => !!userId.value),
  })
}

/**
 * 创建用户
 */
export function useCreateUser() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (data: CreateUserRequest) => {
      const response = await usersApi.create(data)
      if (response.success && response.data) {
        return response.data.user
      }
      throw new Error(response.error || '创建用户失败')
    },
    onSuccess: (newUser) => {
      queryClient.invalidateQueries({ queryKey: userQueryKeys.all })
      queryClient.setQueryData<UserInfo[]>(userQueryKeys.all, (old) =>
        old ? [...old, newUser] : [newUser]
      )
    },
  })
}

/**
 * 更新用户
 */
export function useUpdateUser() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ userId, data }: { userId: string; data: UpdateUserRequest }) => {
      const response = await usersApi.update(userId, data)
      if (response.success && response.data) {
        return response.data.user
      }
      throw new Error(response.error || '更新用户失败')
    },
    onSuccess: (updatedUser, { userId }) => {
      queryClient.setQueryData(userQueryKeys.detail(userId), (old: { user: UserInfo; instances: InstanceInfo[] } | undefined) =>
        old ? { ...old, user: updatedUser } : undefined
      )
      queryClient.setQueryData<UserInfo[]>(userQueryKeys.all, (old) =>
        old?.map((u) => (u.id === userId ? updatedUser : u))
      )
    },
  })
}

/**
 * 删除用户
 */
export function useDeleteUser() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (userId: string) => {
      const response = await usersApi.delete(userId)
      if (!response.success) {
        throw new Error(response.error || '删除用户失败')
      }
    },
    onSuccess: (_, userId) => {
      queryClient.removeQueries({ queryKey: userQueryKeys.detail(userId) })
      queryClient.setQueryData<UserInfo[]>(userQueryKeys.all, (old) =>
        old?.filter((u) => u.id !== userId)
      )
    },
  })
}

/**
 * 激活/停用用户
 */
export function useSetUserActive() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ userId, isActive }: { userId: string; isActive: boolean }) => {
      const response = await usersApi.setActive(userId, isActive)
      if (!response.success) {
        throw new Error(response.error || '操作失败')
      }
      return isActive
    },
    onSuccess: (isActive, { userId }) => {
      queryClient.setQueryData<UserInfo[]>(userQueryKeys.all, (old) =>
        old?.map((u) => (u.id === userId ? { ...u, is_active: isActive } : u))
      )
    },
  })
}

/**
 * 重置用户密码
 */
export function useResetPassword() {
  return useMutation({
    mutationFn: async ({ userId, newPassword }: { userId: string; newPassword: string }) => {
      const response = await usersApi.resetPassword(userId, newPassword)
      if (!response.success) {
        throw new Error(response.error || '重置密码失败')
      }
    },
  })
}

/**
 * 获取用户的实例列表
 */
export function useUserInstances(userId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      userId.value ? userQueryKeys.instances(userId.value) : ['admin', 'users', null, 'instances']
    ),
    queryFn: async () => {
      if (!userId.value) throw new Error('用户 ID 不能为空')
      const response = await usersApi.getInstances(userId.value)
      if (response.success && response.data) {
        return response.data.instances
      }
      throw new Error(response.error || '获取用户实例失败')
    },
    enabled: computed(() => !!userId.value),
  })
}

/**
 * 分配实例给用户
 */
export function useAssignInstances() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ userId, instanceIds }: { userId: string; instanceIds: string[] }) => {
      const response = await usersApi.assignInstances(userId, instanceIds)
      if (!response.success) {
        throw new Error(response.error || '分配实例失败')
      }
    },
    onSuccess: (_, { userId }) => {
      queryClient.invalidateQueries({ queryKey: userQueryKeys.instances(userId) })
      queryClient.invalidateQueries({ queryKey: userQueryKeys.detail(userId) })
    },
  })
}

/**
 * 取消实例分配
 */
export function useUnassignInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ userId, instanceId }: { userId: string; instanceId: string }) => {
      const response = await usersApi.unassignInstance(userId, instanceId)
      if (!response.success) {
        throw new Error(response.error || '取消分配失败')
      }
    },
    onSuccess: (_, { userId }) => {
      queryClient.invalidateQueries({ queryKey: userQueryKeys.instances(userId) })
      queryClient.invalidateQueries({ queryKey: userQueryKeys.detail(userId) })
    },
  })
}
