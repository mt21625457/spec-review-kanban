<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { PageHeader } from '@/components/layout'
import { Card, Button, Badge, Input, Loading } from '@/components/ui'
import {
  useAdminInstances,
  useDeleteInstance,
  useStartInstance,
  useStopInstance,
  useRestartInstance,
} from '@/composables/useInstances'
import type { InstanceInfo } from '@/types'
import CreateInstanceDialog from '@/components/admin/CreateInstanceDialog.vue'

const { t } = useI18n()
const router = useRouter()

const { data: instances, isLoading, refetch } = useAdminInstances()
const deleteInstanceMutation = useDeleteInstance()
const startMutation = useStartInstance()
const stopMutation = useStopInstance()
const restartMutation = useRestartInstance()

const searchQuery = ref('')
const showCreateDialog = ref(false)

const filteredInstances = computed(() => {
  if (!instances.value) return []
  if (!searchQuery.value) return instances.value
  const query = searchQuery.value.toLowerCase()
  return instances.value.filter(
    (i) =>
      i.name.toLowerCase().includes(query) ||
      i.description?.toLowerCase().includes(query)
  )
})

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

const formatErrorTime = (dateStr: string | null) => {
  if (!dateStr) return ''
  return new Date(dateStr).toLocaleString('zh-CN')
}

const handleStart = async (instance: InstanceInfo) => {
  try {
    await startMutation.mutateAsync(instance.id)
    // 启动后刷新列表以获取最新状态
    refetch()
  } catch (error) {
    console.error('启动实例失败:', error)
    // 刷新列表以显示错误信息
    refetch()
  }
}

const handleStop = async (instance: InstanceInfo) => {
  if (!confirm(`确定要停止实例 "${instance.name}" 吗？`)) return
  try {
    await stopMutation.mutateAsync(instance.id)
  } catch (error) {
    console.error('停止实例失败:', error)
  }
}

const handleRestart = async (instance: InstanceInfo) => {
  try {
    await restartMutation.mutateAsync(instance.id)
  } catch (error) {
    console.error('重启实例失败:', error)
  }
}

const handleDelete = async (instance: InstanceInfo) => {
  if (!confirm(`确定要删除实例 "${instance.name}" 吗？此操作不可撤销。`)) return
  try {
    await deleteInstanceMutation.mutateAsync(instance.id)
  } catch (error) {
    console.error('删除实例失败:', error)
  }
}

const goToDetail = (instance: InstanceInfo) => {
  router.push(`/admin/instances/${instance.id}`)
}
</script>

<template>
  <div>
    <PageHeader :title="t('admin.instances.title')">
      <template #actions>
        <Button variant="primary" @click="showCreateDialog = true">
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t('admin.instances.create') }}
        </Button>
      </template>
    </PageHeader>

    <!-- 搜索和过滤 -->
    <div class="mb-6 flex items-center gap-4">
      <div class="flex-1 max-w-md">
        <Input
          v-model="searchQuery"
          :placeholder="t('admin.instances.searchPlaceholder')"
          class="w-full"
        />
      </div>
      <Button variant="ghost" @click="refetch()">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </Button>
    </div>

    <!-- 实例列表 -->
    <div v-if="isLoading" class="py-12">
      <Loading />
    </div>

    <div v-else-if="!filteredInstances.length" class="py-12 text-center text-text-muted">
      {{ t('admin.instances.empty') }}
    </div>

    <div v-else class="grid gap-4">
      <Card
        v-for="instance in filteredInstances"
        :key="instance.id"
        class="hover:shadow-md transition-shadow cursor-pointer"
        @click="goToDetail(instance)"
      >
        <div class="flex items-center gap-4">
          <!-- 状态指示器 -->
          <div
            class="w-12 h-12 rounded-lg flex items-center justify-center"
            :class="{
              'bg-success/20 text-success': instance.status === 'running',
              'bg-text-muted/20 text-text-muted': instance.status === 'stopped',
              'bg-warning/20 text-warning': ['starting', 'stopping'].includes(instance.status),
              'bg-error/20 text-error': instance.status === 'error',
            }"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
            </svg>
          </div>

          <!-- 实例信息 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-text-primary">{{ instance.name }}</span>
              <Badge :variant="getStatusBadge(instance.status)">
                {{ getStatusText(instance.status) }}
              </Badge>
              <Badge v-if="instance.auto_start" variant="brand" size="sm">
                自动启动
              </Badge>
            </div>
            <div v-if="instance.description" class="text-sm text-text-muted mt-0.5 truncate">
              {{ instance.description }}
            </div>
            <!-- 错误信息显示 -->
            <div
              v-if="instance.status === 'error' && instance.last_error"
              class="mt-1.5 p-2 bg-error/10 border border-error/20 rounded text-xs text-error"
            >
              <div class="font-medium">错误信息:</div>
              <div class="mt-0.5 break-all">{{ instance.last_error }}</div>
              <div v-if="instance.last_error_at" class="mt-1 text-error/70">
                {{ formatErrorTime(instance.last_error_at) }}
              </div>
            </div>
            <div v-else class="text-xs text-text-muted mt-1">
              端口: {{ instance.port }}
              <span v-if="instance.user_count !== undefined">
                · {{ instance.user_count }} 个用户
              </span>
              <span v-if="instance.max_users">
                / {{ instance.max_users }} 最大
              </span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2" @click.stop>
            <!-- 启动/重试按钮 (stopped 或 error 状态可用) -->
            <Button
              v-if="instance.status === 'stopped' || instance.status === 'error'"
              variant="ghost"
              size="sm"
              :title="instance.status === 'error' ? '重试启动' : '启动'"
              :loading="startMutation.isPending.value"
              @click="handleStart(instance)"
            >
              <svg class="w-4 h-4 text-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </Button>
            <!-- 停止按钮 -->
            <Button
              v-if="instance.status === 'running'"
              variant="ghost"
              size="sm"
              title="停止"
              :loading="stopMutation.isPending.value"
              @click="handleStop(instance)"
            >
              <svg class="w-4 h-4 text-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
              </svg>
            </Button>
            <Button
              v-if="instance.status === 'running'"
              variant="ghost"
              size="sm"
              title="重启"
              :loading="restartMutation.isPending.value"
              @click="handleRestart(instance)"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </Button>
            <!-- 删除 -->
            <Button
              variant="ghost"
              size="sm"
              title="删除"
              @click="handleDelete(instance)"
            >
              <svg class="w-4 h-4 text-error" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </Button>
          </div>
        </div>
      </Card>
    </div>

    <!-- 创建实例对话框 -->
    <CreateInstanceDialog v-model:open="showCreateDialog" />
  </div>
</template>
