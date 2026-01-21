/**
 * useInstances - 实例管理 composable (管理员)
 */
import { computed, type Ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  instancesApi,
  type CreateInstanceRequest,
  type UpdateInstanceRequest,
  type SetAgentConfigRequest,
  type AgentConfig,
} from '@/lib/api'
import type { InstanceInfo, UserInfo } from '@/types'

// Query keys
export const instanceQueryKeys = {
  all: ['admin', 'instances'] as const,
  detail: (id: string) => ['admin', 'instances', id] as const,
  users: (id: string) => ['admin', 'instances', id, 'users'] as const,
  agents: (id: string) => ['admin', 'instances', id, 'agents'] as const,
  health: (id: string) => ['admin', 'instances', id, 'health'] as const,
}

/**
 * 获取所有实例列表
 */
export function useAdminInstances() {
  return useQuery({
    queryKey: instanceQueryKeys.all,
    queryFn: async () => {
      const response = await instancesApi.list()
      if (response.success && response.data) {
        return response.data.instances
      }
      throw new Error(response.error || '获取实例列表失败')
    },
    staleTime: 30 * 1000,
  })
}

/**
 * 获取实例详情
 */
export function useInstance(instanceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      instanceId.value ? instanceQueryKeys.detail(instanceId.value) : ['admin', 'instances', null]
    ),
    queryFn: async () => {
      if (!instanceId.value) throw new Error('实例 ID 不能为空')
      const response = await instancesApi.get(instanceId.value)
      if (response.success && response.data) {
        return response.data
      }
      throw new Error(response.error || '获取实例详情失败')
    },
    enabled: computed(() => !!instanceId.value),
  })
}

/**
 * 创建实例
 */
export function useCreateInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (data: CreateInstanceRequest) => {
      const response = await instancesApi.create(data)
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '创建实例失败')
    },
    onSuccess: (newInstance) => {
      queryClient.invalidateQueries({ queryKey: instanceQueryKeys.all })
      queryClient.setQueryData<InstanceInfo[]>(instanceQueryKeys.all, (old) =>
        old ? [...old, newInstance] : [newInstance]
      )
    },
  })
}

/**
 * 更新实例
 */
export function useUpdateInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ instanceId, data }: { instanceId: string; data: UpdateInstanceRequest }) => {
      const response = await instancesApi.update(instanceId, data)
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '更新实例失败')
    },
    onSuccess: (updatedInstance, { instanceId }) => {
      queryClient.setQueryData(
        instanceQueryKeys.detail(instanceId),
        (old: { instance: InstanceInfo; users: UserInfo[]; agents: AgentConfig[] } | undefined) =>
          old ? { ...old, instance: updatedInstance } : undefined
      )
      queryClient.setQueryData<InstanceInfo[]>(instanceQueryKeys.all, (old) =>
        old?.map((i) => (i.id === instanceId ? updatedInstance : i))
      )
    },
  })
}

/**
 * 删除实例
 */
export function useDeleteInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (instanceId: string) => {
      const response = await instancesApi.delete(instanceId)
      if (!response.success) {
        throw new Error(response.error || '删除实例失败')
      }
    },
    onSuccess: (_, instanceId) => {
      queryClient.removeQueries({ queryKey: instanceQueryKeys.detail(instanceId) })
      queryClient.setQueryData<InstanceInfo[]>(instanceQueryKeys.all, (old) =>
        old?.filter((i) => i.id !== instanceId)
      )
    },
  })
}

/**
 * 启动实例
 */
export function useStartInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (instanceId: string) => {
      const response = await instancesApi.start(instanceId)
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '启动实例失败')
    },
    onSuccess: (updatedInstance, instanceId) => {
      queryClient.setQueryData<InstanceInfo[]>(instanceQueryKeys.all, (old) =>
        old?.map((i) => (i.id === instanceId ? updatedInstance : i))
      )
    },
  })
}

/**
 * 停止实例
 */
export function useStopInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (instanceId: string) => {
      const response = await instancesApi.stop(instanceId)
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '停止实例失败')
    },
    onSuccess: (updatedInstance, instanceId) => {
      queryClient.setQueryData<InstanceInfo[]>(instanceQueryKeys.all, (old) =>
        old?.map((i) => (i.id === instanceId ? updatedInstance : i))
      )
    },
  })
}

/**
 * 重启实例
 */
export function useRestartInstance() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (instanceId: string) => {
      const response = await instancesApi.restart(instanceId)
      if (response.success && response.data) {
        return response.data.instance
      }
      throw new Error(response.error || '重启实例失败')
    },
    onSuccess: (updatedInstance, instanceId) => {
      queryClient.setQueryData<InstanceInfo[]>(instanceQueryKeys.all, (old) =>
        old?.map((i) => (i.id === instanceId ? updatedInstance : i))
      )
    },
  })
}

/**
 * 实例健康检查（管理员）
 */
export function useAdminInstanceHealth(instanceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      instanceId.value ? instanceQueryKeys.health(instanceId.value) : ['admin', 'instances', null, 'health']
    ),
    queryFn: async () => {
      if (!instanceId.value) throw new Error('实例 ID 不能为空')
      const response = await instancesApi.health(instanceId.value)
      if (response.success && response.data) {
        return response.data.health_status
      }
      throw new Error(response.error || '健康检查失败')
    },
    enabled: computed(() => !!instanceId.value),
    refetchInterval: 30 * 1000,
  })
}

/**
 * 获取实例用户
 */
export function useInstanceUsers(instanceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      instanceId.value ? instanceQueryKeys.users(instanceId.value) : ['admin', 'instances', null, 'users']
    ),
    queryFn: async () => {
      if (!instanceId.value) throw new Error('实例 ID 不能为空')
      const response = await instancesApi.getUsers(instanceId.value)
      if (response.success && response.data) {
        return response.data.users
      }
      throw new Error(response.error || '获取实例用户失败')
    },
    enabled: computed(() => !!instanceId.value),
  })
}

/**
 * 获取实例智能体配置
 */
export function useInstanceAgents(instanceId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      instanceId.value ? instanceQueryKeys.agents(instanceId.value) : ['admin', 'instances', null, 'agents']
    ),
    queryFn: async () => {
      if (!instanceId.value) throw new Error('实例 ID 不能为空')
      const response = await instancesApi.getAgents(instanceId.value)
      if (response.success && response.data) {
        return response.data.agents
      }
      throw new Error(response.error || '获取智能体配置失败')
    },
    enabled: computed(() => !!instanceId.value),
  })
}

/**
 * 设置智能体配置
 */
export function useSetAgentConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({
      instanceId,
      agentType,
      data,
    }: {
      instanceId: string
      agentType: string
      data: SetAgentConfigRequest
    }) => {
      const response = await instancesApi.setAgentConfig(instanceId, agentType, data)
      if (response.success && response.data) {
        return response.data.agent
      }
      throw new Error(response.error || '设置智能体配置失败')
    },
    onSuccess: (_, { instanceId }) => {
      queryClient.invalidateQueries({ queryKey: instanceQueryKeys.agents(instanceId) })
      queryClient.invalidateQueries({ queryKey: instanceQueryKeys.detail(instanceId) })
    },
  })
}

/**
 * 测试智能体连接
 */
export function useTestAgentConnection() {
  return useMutation({
    mutationFn: async ({ instanceId, agentType }: { instanceId: string; agentType: string }) => {
      const response = await instancesApi.testAgentConnection(instanceId, agentType)
      if (response.success && response.data) {
        return response.data.connection_ok
      }
      throw new Error(response.error || '测试连接失败')
    },
  })
}
