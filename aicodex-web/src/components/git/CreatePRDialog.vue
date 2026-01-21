<script setup lang="ts">
/**
 * CreatePRDialog - Dialog for creating pull requests from workspace changes
 */
import { ref, computed, watch, toRef } from 'vue'
import { useUiStore } from '@/stores'
import {
  useWorkspace,
  useWorkspaceRepos,
  useWorkspaceBranchStatus,
  useCreatePR,
} from '@/composables/useWorkspaces'
import Dialog from '@/components/ui/Dialog.vue'
import Button from '@/components/ui/Button.vue'
import Loading from '@/components/ui/Loading.vue'

const uiStore = useUiStore()

// Get workspace ID from dialog data
const workspaceId = computed(() => {
  if (uiStore.activeDialog === 'createPR' && uiStore.dialogData) {
    return (uiStore.dialogData as { workspaceId: string }).workspaceId
  }
  return null
})

const workspaceIdRef = toRef({ value: workspaceId.value }, 'value')
watch(workspaceId, (id) => {
  workspaceIdRef.value = id
})

const { data: workspace } = useWorkspace(workspaceIdRef)
const { data: repos, isLoading: reposLoading } = useWorkspaceRepos(workspaceIdRef)
const { data: branchStatus, isLoading: statusLoading } = useWorkspaceBranchStatus(workspaceIdRef)

const createPRMutation = useCreatePR()

// Form state
const selectedRepoId = ref<string | null>(null)
const prTitle = ref('')
const prBody = ref('')

// Computed
const isOpen = computed(() => uiStore.activeDialog === 'createPR')
const isLoading = computed(() => reposLoading.value || statusLoading.value)
const isCreating = computed(() => createPRMutation.isPending.value)

const selectedBranchStatus = computed(() =>
  branchStatus.value?.find((s) => s.repo_id === selectedRepoId.value)
)

const canCreatePR = computed(() => {
  if (!selectedRepoId.value) return false
  if (!prTitle.value.trim()) return false
  const status = selectedBranchStatus.value
  // Can create PR if there are commits ahead
  return !status || status.ahead_count > 0
})

const hasUnpushedChanges = computed(() => {
  const status = selectedBranchStatus.value
  return status && status.ahead_count > 0
})

// Auto-select first repo
watch(repos, (newRepos) => {
  if (newRepos?.length && !selectedRepoId.value) {
    selectedRepoId.value = newRepos[0].id
  }
}, { immediate: true })

// Generate default PR title from workspace
watch([workspace, isOpen], ([ws, open]) => {
  if (open && ws && !prTitle.value) {
    prTitle.value = `[${ws.branch}] Changes from workspace`
  }
}, { immediate: true })

// Methods
const handleClose = () => {
  uiStore.closeDialog()
  selectedRepoId.value = null
  prTitle.value = ''
  prBody.value = ''
}

const handleCreatePR = async () => {
  if (!workspaceId.value || !selectedRepoId.value || !canCreatePR.value) return

  try {
    await createPRMutation.mutateAsync({
      workspaceId: workspaceId.value,
      repoId: selectedRepoId.value,
      title: prTitle.value.trim(),
      body: prBody.value.trim(),
    })
    handleClose()
  } catch (error) {
    console.error('Create PR failed:', error)
  }
}
</script>

<template>
  <Dialog :open="isOpen" title="创建 Pull Request" size="lg" @update:open="handleClose">
    <!-- Loading -->
    <div v-if="isLoading" class="flex justify-center py-8">
      <Loading />
    </div>

    <!-- Content -->
    <div v-else class="space-y-4">
      <!-- Workspace Info -->
      <div v-if="workspace" class="p-3 bg-bg-secondary rounded-lg">
        <div class="text-sm">
          <span class="text-text-muted">分支:</span>
          <span class="text-text-primary font-mono ml-2">{{ workspace.branch }}</span>
        </div>
      </div>

      <!-- Repository Selection -->
      <div>
        <label class="block text-sm font-medium text-text-secondary mb-2">
          目标仓库
        </label>
        <div class="space-y-2">
          <label
            v-for="repo in repos"
            :key="repo.id"
            class="flex items-center gap-3 p-3 border rounded-lg cursor-pointer transition-colors"
            :class="{
              'border-brand bg-brand/5': selectedRepoId === repo.id,
              'border-border-normal hover:border-border-hover': selectedRepoId !== repo.id,
            }"
          >
            <input
              v-model="selectedRepoId"
              type="radio"
              :value="repo.id"
              class="text-brand focus:ring-brand"
            />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-text-primary">
                {{ repo.display_name }}
              </div>
              <div class="text-xs text-text-muted truncate">
                目标分支: {{ repo.target_branch }}
              </div>
            </div>

            <!-- Branch Status -->
            <div v-if="branchStatus?.find(s => s.repo_id === repo.id)" class="text-xs">
              <span
                v-if="branchStatus.find(s => s.repo_id === repo.id)?.ahead_count"
                class="text-green-600 dark:text-green-400"
              >
                ↑{{ branchStatus.find(s => s.repo_id === repo.id)?.ahead_count }} commits
              </span>
            </div>
          </label>
        </div>
      </div>

      <!-- PR Title -->
      <div>
        <label class="block text-sm font-medium text-text-secondary mb-2">
          PR 标题 <span class="text-red-500">*</span>
        </label>
        <input
          v-model="prTitle"
          type="text"
          class="w-full px-3 py-2 rounded-lg border border-border-normal bg-bg-primary text-text-primary placeholder:text-text-muted focus:border-brand focus:ring-1 focus:ring-brand outline-none transition-colors"
          placeholder="输入 Pull Request 标题"
        />
      </div>

      <!-- PR Body -->
      <div>
        <label class="block text-sm font-medium text-text-secondary mb-2">
          PR 描述
        </label>
        <textarea
          v-model="prBody"
          rows="6"
          class="w-full px-3 py-2 rounded-lg border border-border-normal bg-bg-primary text-text-primary placeholder:text-text-muted focus:border-brand focus:ring-1 focus:ring-brand outline-none transition-colors resize-none"
          placeholder="输入 Pull Request 描述（可选）"
        />
      </div>

      <!-- Unpushed Changes Warning -->
      <div v-if="hasUnpushedChanges" class="p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
        <div class="flex items-center gap-2 text-yellow-600 dark:text-yellow-400">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-sm font-medium">有未推送的提交</span>
        </div>
        <p class="text-sm text-yellow-600/80 dark:text-yellow-400/80 mt-1">
          创建 PR 前需要先推送本地提交到远程仓库。
        </p>
      </div>

      <!-- Error -->
      <div v-if="createPRMutation.error.value" class="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
        <p class="text-sm text-red-600 dark:text-red-400">
          {{ createPRMutation.error.value.message || '创建 PR 失败' }}
        </p>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" :disabled="isCreating" @click="handleClose">
          取消
        </Button>
        <Button
          variant="primary"
          :loading="isCreating"
          :disabled="!canCreatePR || isCreating"
          @click="handleCreatePR"
        >
          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
          </svg>
          创建 Pull Request
        </Button>
      </div>
    </template>
  </Dialog>
</template>
