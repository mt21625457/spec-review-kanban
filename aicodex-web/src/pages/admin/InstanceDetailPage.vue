<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { PageHeader } from '@/components/layout'
import { Card, Button, Badge, Input, Loading } from '@/components/ui'
import {
  useInstance,
  useUpdateInstance,
  useStartInstance,
  useStopInstance,
  useRestartInstance,
  useAdminInstanceHealth,
  useInstanceUsers,
  useInstanceAgents,
  useSetAgentConfig,
  useTestAgentConnection,
} from '@/composables/useInstances'
import type { InstanceInfo } from '@/types'
import type { AgentConfig } from '@/lib/api'

const route = useRoute()
const router = useRouter()

const instanceId = ref<string | null>(route.params.id as string)

watch(() => route.params.id, (newId) => {
  instanceId.value = newId as string
})

const { data: instanceData, isLoading } = useInstance(instanceId)
const { data: healthData } = useAdminInstanceHealth(instanceId)
const { data: users } = useInstanceUsers(instanceId)
const { data: agents, refetch: refetchAgents } = useInstanceAgents(instanceId)

const updateMutation = useUpdateInstance()
const startMutation = useStartInstance()
const stopMutation = useStopInstance()
const restartMutation = useRestartInstance()
const setAgentConfigMutation = useSetAgentConfig()
const testConnectionMutation = useTestAgentConnection()

const instance = computed(() => instanceData.value?.instance)
const activeTab = ref<'info' | 'users' | 'agents'>('info')

// 编辑表单
const editMode = ref(false)
const editForm = ref({
  name: '',
  description: '',
  auto_start: true,
  max_users: null as number | null,
})

watch(instance, (inst) => {
  if (inst) {
    editForm.value = {
      name: inst.name,
      description: inst.description || '',
      auto_start: inst.auto_start,
      max_users: inst.max_users,
    }
  }
})

// AI 智能体配置
const agentTypes = [
  { type: 'claude_code', name: 'Claude Code', description: 'Anthropic Claude 智能体' },
  { type: 'codex_cli', name: 'Codex CLI', description: 'OpenAI Codex 智能体' },
  { type: 'gemini_cli', name: 'Gemini CLI', description: 'Google Gemini 智能体' },
  { type: 'opencode', name: 'OpenCode', description: '开源代码智能体' },
]

const editingAgent = ref<string | null>(null)
const agentForm = ref({
  is_enabled: false,
  api_key: '',
  rate_limit_rpm: 60,
  config_json: '{}',
})

const getAgentConfig = (agentType: string): AgentConfig | undefined => {
  return agents.value?.find(a => a.agent_type === agentType)
}

const startEditAgent = (agentType: string) => {
  const config = getAgentConfig(agentType)
  editingAgent.value = agentType
  agentForm.value = {
    is_enabled: config?.is_enabled ?? false,
    api_key: '', // 不显示已保存的 API key
    rate_limit_rpm: config?.rate_limit_rpm ?? 60,
    config_json: config?.config_json ?? '{}',
  }
}

const cancelEditAgent = () => {
  editingAgent.value = null
}

const saveAgentConfig = async () => {
  if (!instanceId.value || !editingAgent.value) return

  try {
    await setAgentConfigMutation.mutateAsync({
      instanceId: instanceId.value,
      agentType: editingAgent.value,
      data: {
        is_enabled: agentForm.value.is_enabled,
        api_key: agentForm.value.api_key || undefined,
        rate_limit_rpm: agentForm.value.rate_limit_rpm,
        config_json: agentForm.value.config_json,
      },
    })
    editingAgent.value = null
    refetchAgents()
  } catch (error) {
    console.error('保存智能体配置失败:', error)
  }
}

const testConnection = async (agentType: string) => {
  if (!instanceId.value) return

  try {
    const isOk = await testConnectionMutation.mutateAsync({
      instanceId: instanceId.value,
      agentType,
    })
    alert(isOk ? '连接测试成功!' : '连接测试失败')
  } catch (error) {
    alert('连接测试失败: ' + (error instanceof Error ? error.message : '未知错误'))
  }
}

type BadgeVariant = 'default' | 'pending' | 'running' | 'completed' | 'failed' | 'cancelled' | 'success' | 'secondary' | 'brand' | 'danger' | 'warning'

const getStatusBadge = (status: InstanceInfo['status']): BadgeVariant => {
  const map: Record<string, BadgeVariant> = {
    running: 'success',
    stopped: 'secondary',
    starting: 'warning',
    stopping: 'warning',
    error: 'danger',
  }
  return map[status] || 'secondary'
}

const getStatusText = (status: InstanceInfo['status']) => {
  const map: Record<string, string> = {
    running: '运行中',
    stopped: '已停止',
    starting: '启动中',
    stopping: '停止中',
    error: '错误',
  }
  return map[status] || status
}

const handleStart = async () => {
  if (!instanceId.value) return
  try {
    await startMutation.mutateAsync(instanceId.value)
  } catch (error) {
    console.error('启动实例失败:', error)
  }
}

const handleStop = async () => {
  if (!instanceId.value || !instance.value) return
  if (!confirm(`确定要停止实例 "${instance.value.name}" 吗？`)) return
  try {
    await stopMutation.mutateAsync(instanceId.value)
  } catch (error) {
    console.error('停止实例失败:', error)
  }
}

const handleRestart = async () => {
  if (!instanceId.value) return
  try {
    await restartMutation.mutateAsync(instanceId.value)
  } catch (error) {
    console.error('重启实例失败:', error)
  }
}

const handleSave = async () => {
  if (!instanceId.value) return
  try {
    await updateMutation.mutateAsync({
      instanceId: instanceId.value,
      data: {
        name: editForm.value.name,
        description: editForm.value.description || undefined,
        auto_start: editForm.value.auto_start,
        max_users: editForm.value.max_users || undefined,
      },
    })
    editMode.value = false
  } catch (error) {
    console.error('保存实例失败:', error)
  }
}

const goBack = () => {
  router.push('/admin/instances')
}
</script>

<template>
  <div>
    <!-- 加载状态 -->
    <div v-if="isLoading" class="py-12">
      <Loading />
    </div>

    <template v-else-if="instance">
      <PageHeader :title="instance.name">
        <template #actions>
          <div class="flex items-center gap-2">
            <Button variant="ghost" @click="goBack">
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
              </svg>
              返回
            </Button>
            <Button
              v-if="instance.status === 'stopped'"
              variant="primary"
              :loading="startMutation.isPending.value"
              @click="handleStart"
            >
              启动
            </Button>
            <Button
              v-if="instance.status === 'running'"
              variant="ghost"
              :loading="stopMutation.isPending.value"
              @click="handleStop"
            >
              停止
            </Button>
            <Button
              v-if="instance.status === 'running'"
              variant="ghost"
              :loading="restartMutation.isPending.value"
              @click="handleRestart"
            >
              重启
            </Button>
          </div>
        </template>
      </PageHeader>

      <!-- 状态概览 -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <Card>
          <div class="text-sm text-text-muted mb-1">状态</div>
          <div class="flex items-center gap-2">
            <Badge :variant="getStatusBadge(instance.status)">
              {{ getStatusText(instance.status) }}
            </Badge>
          </div>
        </Card>
        <Card>
          <div class="text-sm text-text-muted mb-1">端口</div>
          <div class="text-lg font-medium text-text-primary">{{ instance.port }}</div>
        </Card>
        <Card>
          <div class="text-sm text-text-muted mb-1">用户数</div>
          <div class="text-lg font-medium text-text-primary">
            {{ users?.length || 0 }}
            <span v-if="instance.max_users" class="text-sm text-text-muted">
              / {{ instance.max_users }}
            </span>
          </div>
        </Card>
        <Card>
          <div class="text-sm text-text-muted mb-1">健康状态</div>
          <div class="flex items-center gap-2">
            <span
              class="w-2 h-2 rounded-full"
              :class="{
                'bg-success': healthData === 'healthy',
                'bg-warning': healthData === 'degraded',
                'bg-error': healthData === 'unhealthy',
                'bg-text-muted': !healthData,
              }"
            />
            <span class="text-text-primary">
              {{ healthData || '未知' }}
            </span>
          </div>
        </Card>
      </div>

      <!-- 标签页切换 -->
      <div class="flex gap-1 mb-6 border-b border-border">
        <button
          class="px-4 py-2 text-sm font-medium transition-colors"
          :class="activeTab === 'info' ? 'text-brand border-b-2 border-brand' : 'text-text-muted hover:text-text-primary'"
          @click="activeTab = 'info'"
        >
          基本信息
        </button>
        <button
          class="px-4 py-2 text-sm font-medium transition-colors"
          :class="activeTab === 'users' ? 'text-brand border-b-2 border-brand' : 'text-text-muted hover:text-text-primary'"
          @click="activeTab = 'users'"
        >
          用户列表
        </button>
        <button
          class="px-4 py-2 text-sm font-medium transition-colors"
          :class="activeTab === 'agents' ? 'text-brand border-b-2 border-brand' : 'text-text-muted hover:text-text-primary'"
          @click="activeTab = 'agents'"
        >
          AI 智能体
        </button>
      </div>

      <!-- 基本信息标签页 -->
      <div v-show="activeTab === 'info'">
        <Card>
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-medium text-text-primary">实例配置</h3>
            <Button
              v-if="!editMode"
              variant="ghost"
              size="sm"
              @click="editMode = true"
            >
              编辑
            </Button>
          </div>

          <div v-if="!editMode" class="space-y-4">
            <div>
              <div class="text-sm text-text-muted">名称</div>
              <div class="text-text-primary">{{ instance.name }}</div>
            </div>
            <div>
              <div class="text-sm text-text-muted">描述</div>
              <div class="text-text-primary">{{ instance.description || '-' }}</div>
            </div>
            <div>
              <div class="text-sm text-text-muted">自动启动</div>
              <div class="text-text-primary">{{ instance.auto_start ? '是' : '否' }}</div>
            </div>
            <div>
              <div class="text-sm text-text-muted">最大用户数</div>
              <div class="text-text-primary">{{ instance.max_users || '不限制' }}</div>
            </div>
            <div>
              <div class="text-sm text-text-muted">创建时间</div>
              <div class="text-text-primary">{{ new Date(instance.created_at).toLocaleString('zh-CN') }}</div>
            </div>
          </div>

          <form v-else class="space-y-4" @submit.prevent="handleSave">
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1.5">名称</label>
              <Input v-model="editForm.name" />
            </div>
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1.5">描述</label>
              <Input v-model="editForm.description" />
            </div>
            <div>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="editForm.auto_start"
                  type="checkbox"
                  class="w-4 h-4 text-brand focus:ring-brand rounded"
                />
                <span class="text-sm text-text-primary">自动启动</span>
              </label>
            </div>
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1.5">最大用户数</label>
              <Input v-model.number="editForm.max_users" type="number" :min="1" placeholder="留空表示不限制" />
            </div>
            <div class="flex justify-end gap-3 pt-4">
              <Button variant="ghost" type="button" @click="editMode = false">取消</Button>
              <Button variant="primary" type="submit" :loading="updateMutation.isPending.value">保存</Button>
            </div>
          </form>
        </Card>
      </div>

      <!-- 用户列表标签页 -->
      <div v-show="activeTab === 'users'">
        <Card>
          <h3 class="text-lg font-medium text-text-primary mb-4">已分配用户</h3>

          <div v-if="!users?.length" class="py-8 text-center text-text-muted">
            暂无用户分配到此实例
          </div>

          <div v-else class="space-y-2">
            <div
              v-for="user in users"
              :key="user.id"
              class="flex items-center gap-3 p-3 rounded-lg bg-surface-hover"
            >
              <div
                class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-medium bg-brand/20 text-brand"
              >
                {{ (user.display_name || user.username).charAt(0).toUpperCase() }}
              </div>
              <div class="flex-1">
                <div class="font-medium text-text-primary">
                  {{ user.display_name || user.username }}
                </div>
                <div class="text-sm text-text-muted">@{{ user.username }}</div>
              </div>
              <Badge :variant="user.role === 'admin' ? 'brand' : 'secondary'">
                {{ user.role === 'admin' ? '管理员' : '用户' }}
              </Badge>
            </div>
          </div>
        </Card>
      </div>

      <!-- AI 智能体标签页 -->
      <div v-show="activeTab === 'agents'">
        <div class="space-y-4">
          <Card v-for="agentType in agentTypes" :key="agentType.type">
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                  <h4 class="font-medium text-text-primary">{{ agentType.name }}</h4>
                  <Badge
                    :variant="getAgentConfig(agentType.type)?.is_enabled ? 'success' : 'secondary'"
                    size="sm"
                  >
                    {{ getAgentConfig(agentType.type)?.is_enabled ? '已启用' : '未启用' }}
                  </Badge>
                </div>
                <p class="text-sm text-text-muted">{{ agentType.description }}</p>

                <!-- 已配置信息 -->
                <div v-if="getAgentConfig(agentType.type) && editingAgent !== agentType.type" class="mt-3 text-sm">
                  <div class="text-text-muted">
                    速率限制: {{ getAgentConfig(agentType.type)?.rate_limit_rpm }} 请求/分钟
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2">
                <Button
                  v-if="editingAgent !== agentType.type"
                  variant="ghost"
                  size="sm"
                  @click="startEditAgent(agentType.type)"
                >
                  配置
                </Button>
                <Button
                  v-if="getAgentConfig(agentType.type)?.is_enabled && editingAgent !== agentType.type"
                  variant="ghost"
                  size="sm"
                  :loading="testConnectionMutation.isPending.value"
                  @click="testConnection(agentType.type)"
                >
                  测试连接
                </Button>
              </div>
            </div>

            <!-- 编辑表单 -->
            <div v-if="editingAgent === agentType.type" class="mt-4 pt-4 border-t border-border">
              <form class="space-y-4" @submit.prevent="saveAgentConfig">
                <div>
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input
                      v-model="agentForm.is_enabled"
                      type="checkbox"
                      class="w-4 h-4 text-brand focus:ring-brand rounded"
                    />
                    <span class="text-sm text-text-primary">启用此智能体</span>
                  </label>
                </div>

                <div>
                  <label class="block text-sm font-medium text-text-primary mb-1.5">
                    API Key
                  </label>
                  <Input
                    v-model="agentForm.api_key"
                    type="password"
                    placeholder="输入新的 API Key（留空保持不变）"
                  />
                </div>

                <div>
                  <label class="block text-sm font-medium text-text-primary mb-1.5">
                    速率限制 (请求/分钟)
                  </label>
                  <Input
                    v-model.number="agentForm.rate_limit_rpm"
                    type="number"
                    :min="1"
                    :max="1000"
                  />
                </div>

                <div>
                  <label class="block text-sm font-medium text-text-primary mb-1.5">
                    额外配置 (JSON)
                  </label>
                  <textarea
                    v-model="agentForm.config_json"
                    class="w-full px-3 py-2 rounded-lg border border-border bg-surface text-text-primary text-sm focus:outline-none focus:ring-2 focus:ring-brand/50"
                    rows="3"
                    placeholder="{}"
                  />
                </div>

                <div class="flex justify-end gap-3">
                  <Button variant="ghost" type="button" @click="cancelEditAgent">取消</Button>
                  <Button variant="primary" type="submit" :loading="setAgentConfigMutation.isPending.value">
                    保存配置
                  </Button>
                </div>
              </form>
            </div>
          </Card>
        </div>
      </div>
    </template>

    <!-- 实例不存在 -->
    <div v-else class="py-12 text-center">
      <p class="text-text-muted mb-4">实例不存在或已被删除</p>
      <Button variant="primary" @click="goBack">返回实例列表</Button>
    </div>
  </div>
</template>
