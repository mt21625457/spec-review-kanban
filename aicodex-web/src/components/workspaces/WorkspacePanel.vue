<script setup lang="ts">
/**
 * WorkspacePanel - Sliding panel for workspace details and execution
 */
import { computed, toRef, ref } from 'vue'
import { useUiStore } from '@/stores'
import {
  useWorkspace,
  useWorkspaceRepos,
  useWorkspaceSessions,
  getWorkspaceStatusInfo,
  useCreateSession,
  useSessionFollowUp,
} from '@/composables/useWorkspaces'
import { useExecutionLogsStream } from '@/lib/websocket'
import Button from '@/components/ui/Button.vue'
import Badge from '@/components/ui/Badge.vue'
import Loading from '@/components/ui/Loading.vue'
import Tabs from '@/components/ui/Tabs.vue'
import TabPanel from '@/components/ui/TabPanel.vue'
import { formatDistanceToNow, format } from 'date-fns'
import { zhCN } from 'date-fns/locale'

const uiStore = useUiStore()

// Fetch workspace details
const workspaceId = toRef(uiStore, 'selectedAttemptId')
const { data: workspace, isLoading, error, refetch } = useWorkspace(workspaceId)
const { data: repos } = useWorkspaceRepos(workspaceId)
const { data: sessions } = useWorkspaceSessions(workspaceId)

// Execution logs stream
const { logs, status: logsStatus, clearLogs } = useExecutionLogsStream(workspaceId)

// Mutations
const createSessionMutation = useCreateSession()
const followUpMutation = useSessionFollowUp()

// Local state
const activeTab = ref('logs')
const followUpPrompt = ref('')

// Computed
const isOpen = computed(() => uiStore.activePanelType === 'attempt' && !!workspaceId.value)
const statusInfo = computed(() => workspace.value ? getWorkspaceStatusInfo(workspace.value) : null)
const isLogsConnected = computed(() => logsStatus.value === 'OPEN')

const latestSession = computed(() => {
  if (!sessions.value?.length) return null
  return sessions.value.sort((a, b) =>
    new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  )[0]
})

const formatDate = (dateStr: string | null) => {
  if (!dateStr) return '-'
  try {
    return format(new Date(dateStr), 'yyyy-MM-dd HH:mm:ss', { locale: zhCN })
  } catch {
    return dateStr
  }
}

const relativeTime = (dateStr: string | null) => {
  if (!dateStr) return '-'
  try {
    return formatDistanceToNow(new Date(dateStr), { addSuffix: true, locale: zhCN })
  } catch {
    return dateStr
  }
}

// Methods
const handleClose = () => {
  uiStore.closePanel()
}

const handleViewDiffs = () => {
  if (workspaceId.value) {
    uiStore.openDiffsPanel(workspaceId.value)
  }
}

const handleStartSession = async () => {
  if (!workspaceId.value) return
  try {
    await createSessionMutation.mutateAsync({
      workspace_id: workspaceId.value,
    })
  } catch (error) {
    console.error('Failed to create session:', error)
  }
}

const handleFollowUp = async () => {
  if (!latestSession.value || !followUpPrompt.value.trim()) return
  try {
    await followUpMutation.mutateAsync({
      sessionId: latestSession.value.id,
      prompt: followUpPrompt.value.trim(),
    })
    followUpPrompt.value = ''
  } catch (error) {
    console.error('Failed to send follow-up:', error)
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="translate-x-full"
      enter-to-class="translate-x-0"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="translate-x-0"
      leave-to-class="translate-x-full"
    >
      <div
        v-if="isOpen"
        class="fixed inset-y-0 right-0 w-full max-w-2xl bg-bg-primary border-l border-border-normal shadow-xl z-40 flex flex-col"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3 border-b border-border-normal">
          <div class="flex items-center gap-3">
            <h2 class="text-lg font-semibold text-text-primary">工作区详情</h2>
            <Badge v-if="statusInfo" :class="statusInfo.color" size="sm">
              {{ statusInfo.label }}
            </Badge>
          </div>
          <div class="flex items-center gap-2">
            <Button variant="secondary" size="sm" @click="handleViewDiffs">
              <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              查看变更
            </Button>
            <button
              type="button"
              class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
              @click="handleClose"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-hidden flex flex-col">
          <!-- Loading -->
          <div v-if="isLoading" class="flex items-center justify-center h-64">
            <Loading />
          </div>

          <!-- Error -->
          <div v-else-if="error" class="p-4 text-center">
            <div class="text-red-500 mb-4">
              <p class="text-sm text-text-secondary">加载工作区失败</p>
            </div>
            <Button variant="secondary" @click="refetch">重试</Button>
          </div>

          <!-- Workspace Details -->
          <template v-else-if="workspace">
            <!-- Info Section -->
            <div class="p-4 border-b border-border-normal">
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="text-text-muted">分支</span>
                  <p class="text-text-primary font-mono">{{ workspace.branch }}</p>
                </div>
                <div>
                  <span class="text-text-muted">创建时间</span>
                  <p class="text-text-primary" :title="formatDate(workspace.created_at)">
                    {{ relativeTime(workspace.created_at) }}
                  </p>
                </div>
                <div v-if="workspace.setup_completed_at">
                  <span class="text-text-muted">初始化完成</span>
                  <p class="text-text-primary" :title="formatDate(workspace.setup_completed_at)">
                    {{ relativeTime(workspace.setup_completed_at) }}
                  </p>
                </div>
                <div v-if="workspace.agent_working_dir">
                  <span class="text-text-muted">工作目录</span>
                  <p class="text-text-primary font-mono text-xs truncate">{{ workspace.agent_working_dir }}</p>
                </div>
              </div>

              <!-- Repositories -->
              <div v-if="repos?.length" class="mt-4">
                <span class="text-text-muted text-sm">关联仓库</span>
                <div class="mt-1 space-y-1">
                  <div
                    v-for="repo in repos"
                    :key="repo.id"
                    class="flex items-center gap-2 text-sm"
                  >
                    <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    <span class="text-text-primary">{{ repo.display_name }}</span>
                    <span class="text-text-muted font-mono text-xs">{{ repo.target_branch }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Tabs -->
            <Tabs v-model="activeTab" :tabs="[
              { name: 'logs', label: '执行日志' },
              { name: 'sessions', label: '会话历史' },
            ]" />

            <!-- Tab Content -->
            <div class="flex-1 overflow-hidden">
              <!-- Logs Tab -->
              <TabPanel name="logs" class="h-full flex flex-col">
                <!-- Logs Status -->
                <div class="flex items-center justify-between px-4 py-2 border-b border-border-normal text-xs">
                  <div class="flex items-center gap-2">
                    <span
                      class="w-2 h-2 rounded-full"
                      :class="{
                        'bg-green-500': isLogsConnected,
                        'bg-gray-400': !isLogsConnected,
                      }"
                    />
                    <span class="text-text-muted">
                      {{ isLogsConnected ? '实时日志' : '日志未连接' }}
                    </span>
                  </div>
                  <button
                    type="button"
                    class="text-text-muted hover:text-text-primary"
                    @click="clearLogs"
                  >
                    清除
                  </button>
                </div>

                <!-- Logs Content -->
                <div class="flex-1 overflow-y-auto p-4 bg-gray-900 font-mono text-xs">
                  <div v-if="!logs.length" class="text-gray-500 text-center py-8">
                    暂无日志
                  </div>
                  <div v-else class="space-y-0.5">
                    <div
                      v-for="(log, index) in logs"
                      :key="index"
                      class="text-gray-300 whitespace-pre-wrap break-all"
                    >
                      {{ log }}
                    </div>
                  </div>
                </div>
              </TabPanel>

              <!-- Sessions Tab -->
              <TabPanel name="sessions" class="h-full overflow-y-auto p-4">
                <div v-if="!sessions?.length" class="text-center py-8 text-text-muted">
                  <p class="text-sm">暂无会话</p>
                  <Button
                    variant="primary"
                    size="sm"
                    class="mt-4"
                    :loading="createSessionMutation.isPending.value"
                    @click="handleStartSession"
                  >
                    开始新会话
                  </Button>
                </div>

                <div v-else class="space-y-4">
                  <!-- Session List -->
                  <div
                    v-for="session in sessions"
                    :key="session.id"
                    class="p-3 border border-border-normal rounded-lg"
                  >
                    <div class="flex items-center justify-between mb-2">
                      <span class="text-sm font-medium text-text-primary">
                        会话 {{ session.id.slice(0, 8) }}
                      </span>
                      <span class="text-xs text-text-muted">
                        {{ relativeTime(session.created_at) }}
                      </span>
                    </div>
                    <div class="text-xs text-text-muted">
                      执行器: {{ session.executor || '默认' }}
                    </div>
                  </div>

                  <!-- Follow-up Input -->
                  <div v-if="latestSession" class="pt-4 border-t border-border-normal">
                    <div class="flex gap-2">
                      <input
                        v-model="followUpPrompt"
                        type="text"
                        class="input flex-1"
                        placeholder="输入跟进指令..."
                        @keydown.enter="handleFollowUp"
                      />
                      <Button
                        variant="primary"
                        :loading="followUpMutation.isPending.value"
                        :disabled="!followUpPrompt.trim()"
                        @click="handleFollowUp"
                      >
                        发送
                      </Button>
                    </div>
                  </div>
                </div>
              </TabPanel>
            </div>
          </template>
        </div>
      </div>
    </Transition>

    <!-- Backdrop -->
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 bg-black/30 z-30"
        @click="handleClose"
      />
    </Transition>
  </Teleport>
</template>
