<script setup lang="ts">
/**
 * GitPushDialog - Dialog for pushing workspace changes to remote
 */
import { ref, computed, watch, toRef } from 'vue'
import { useUiStore } from '@/stores'
import {
  useWorkspace,
  useWorkspaceRepos,
  useWorkspaceBranchStatus,
  usePushWorkspace,
} from '@/composables/useWorkspaces'
import Dialog from '@/components/ui/Dialog.vue'
import Button from '@/components/ui/Button.vue'
import Loading from '@/components/ui/Loading.vue'

const uiStore = useUiStore()

// Get workspace ID from dialog data
const workspaceId = computed(() => {
  if (uiStore.activeDialog === 'gitPush' && uiStore.dialogData) {
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

const pushMutation = usePushWorkspace()

// Form state
const selectedRepoId = ref<string | null>(null)
const forcePush = ref(false)

// Computed
const isOpen = computed(() => uiStore.activeDialog === 'gitPush')
const isLoading = computed(() => reposLoading.value || statusLoading.value)
const isPushing = computed(() => pushMutation.isPending.value)

const selectedBranchStatus = computed(() =>
  branchStatus.value?.find((s) => s.repo_id === selectedRepoId.value)
)

const canPush = computed(() => {
  if (!selectedRepoId.value) return false
  const status = selectedBranchStatus.value
  if (!status) return true
  // Can push if there are commits ahead
  return status.ahead_count > 0 || forcePush.value
})

const hasConflicts = computed(() =>
  selectedBranchStatus.value?.has_conflicts || false
)

// Auto-select first repo
watch(repos, (newRepos) => {
  if (newRepos?.length && !selectedRepoId.value) {
    selectedRepoId.value = newRepos[0].id
  }
}, { immediate: true })

// Methods
const handleClose = () => {
  uiStore.closeDialog()
  selectedRepoId.value = null
  forcePush.value = false
}

const handlePush = async () => {
  if (!workspaceId.value || !selectedRepoId.value || !canPush.value) return

  try {
    await pushMutation.mutateAsync({
      workspaceId: workspaceId.value,
      repoId: selectedRepoId.value,
      force: forcePush.value,
    })
    handleClose()
  } catch (error) {
    console.error('Push failed:', error)
  }
}
</script>

<template>
  <Dialog :open="isOpen" title="推送到远程" size="md" @update:open="handleClose">
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
          选择仓库
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
                {{ repo.target_branch }}
              </div>
            </div>

            <!-- Branch Status -->
            <div v-if="branchStatus?.find(s => s.repo_id === repo.id)" class="text-xs">
              <span
                v-if="branchStatus.find(s => s.repo_id === repo.id)?.ahead_count"
                class="text-green-600 dark:text-green-400"
              >
                ↑{{ branchStatus.find(s => s.repo_id === repo.id)?.ahead_count }}
              </span>
              <span
                v-if="branchStatus.find(s => s.repo_id === repo.id)?.behind_count"
                class="text-yellow-600 dark:text-yellow-400 ml-1"
              >
                ↓{{ branchStatus.find(s => s.repo_id === repo.id)?.behind_count }}
              </span>
            </div>
          </label>
        </div>
      </div>

      <!-- Conflict Warning -->
      <div v-if="hasConflicts" class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <div class="flex items-center gap-2 text-red-600 dark:text-red-400">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <span class="text-sm font-medium">存在冲突</span>
        </div>
        <p class="text-sm text-red-600/80 dark:text-red-400/80 mt-1">
          此分支与远程有冲突，建议先解决冲突后再推送。
        </p>
      </div>

      <!-- Force Push Option -->
      <div v-if="selectedBranchStatus?.behind_count" class="flex items-center gap-2">
        <input
          id="forcePush"
          v-model="forcePush"
          type="checkbox"
          class="rounded border-border-normal text-brand focus:ring-brand"
        />
        <label for="forcePush" class="text-sm text-text-secondary">
          强制推送 (覆盖远程变更)
        </label>
      </div>

      <!-- Push Error -->
      <div v-if="pushMutation.error.value" class="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
        <p class="text-sm text-red-600 dark:text-red-400">
          {{ pushMutation.error.value.message || '推送失败' }}
        </p>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" :disabled="isPushing" @click="handleClose">
          取消
        </Button>
        <Button
          variant="primary"
          :loading="isPushing"
          :disabled="!canPush || isPushing"
          @click="handlePush"
        >
          <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
          </svg>
          {{ forcePush ? '强制推送' : '推送' }}
        </Button>
      </div>
    </template>
  </Dialog>
</template>
