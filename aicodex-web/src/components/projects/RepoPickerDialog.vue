<script setup lang="ts">
/**
 * RepoPickerDialog - Dialog for selecting or creating a git repository
 * Two options:
 * 1. From Git Repository - Select existing repo
 * 2. Create New Repository - Initialize new repo
 */
import { ref, computed, watch } from 'vue'
import { fileSystemApi, reposApi, type DirectoryEntry } from '@/lib/api'
import Dialog from '@/components/ui/Dialog.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Loading from '@/components/ui/Loading.vue'
import type { Repo } from '@/types'

const props = defineProps<{
  open: boolean
  title?: string
  description?: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  select: [repo: Repo]
  cancel: []
}>()

type Stage = 'options' | 'existing' | 'new'

const stage = ref<Stage>('options')
const error = ref('')
const isWorking = ref(false)

// Stage: existing
const allRepos = ref<DirectoryEntry[]>([])
const reposLoading = ref(false)
const showMoreRepos = ref(false)
const loadingDuration = ref(0)
const hasSearched = ref(false)

// Stage: new
const repoName = ref('')
const parentPath = ref('')

// Reset state when dialog opens
watch(
  () => props.open,
  (open) => {
    if (open) {
      stage.value = 'options'
      error.value = ''
      allRepos.value = []
      showMoreRepos.value = false
      repoName.value = ''
      parentPath.value = ''
      loadingDuration.value = 0
      hasSearched.value = false
    }
  }
)

// Load repos when entering existing stage
watch(stage, (newStage) => {
  if (newStage === 'existing' && allRepos.value.length === 0 && !reposLoading.value && !hasSearched.value) {
    loadRecentRepos()
  }
})

// Track loading duration
let loadingInterval: ReturnType<typeof setInterval> | null = null
watch(reposLoading, (loading) => {
  if (loading) {
    loadingDuration.value = 0
    loadingInterval = setInterval(() => {
      loadingDuration.value++
    }, 1000)
  } else if (loadingInterval) {
    clearInterval(loadingInterval)
    loadingInterval = null
  }
})

const displayedRepos = computed(() => {
  if (showMoreRepos.value) return allRepos.value
  return allRepos.value.slice(0, 3)
})

const loadRecentRepos = async () => {
  reposLoading.value = true
  error.value = ''
  try {
    const repos = await fileSystemApi.listGitRepos()
    allRepos.value = repos
  } catch (err) {
    error.value = '加载仓库列表失败'
    console.error('Failed to load repos:', err)
  } finally {
    reposLoading.value = false
    hasSearched.value = true
  }
}

const registerAndReturn = async (path: string) => {
  isWorking.value = true
  error.value = ''
  try {
    const repo = await reposApi.register({ path })
    emit('select', repo)
    emit('update:open', false)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '注册仓库失败'
  } finally {
    isWorking.value = false
  }
}

const handleSelectRepo = (repo: DirectoryEntry) => {
  if (!isWorking.value) {
    registerAndReturn(repo.path)
  }
}

const handleCreateRepo = async () => {
  if (!repoName.value.trim()) {
    error.value = '请输入仓库名称'
    return
  }

  isWorking.value = true
  error.value = ''
  try {
    const repo = await reposApi.init({
      parent_path: parentPath.value.trim() || '.',
      folder_name: repoName.value.trim(),
    })
    emit('select', repo)
    emit('update:open', false)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '创建仓库失败'
  } finally {
    isWorking.value = false
  }
}

const handleClose = () => {
  if (!isWorking.value) {
    emit('cancel')
    emit('update:open', false)
  }
}

const goBack = () => {
  stage.value = 'options'
  error.value = ''
}
</script>

<template>
  <Dialog
    :open="open"
    :title="title || '选择仓库'"
    @update:open="handleClose"
  >
    <div class="space-y-4">
      <!-- Stage: Options -->
      <template v-if="stage === 'options'">
        <p class="text-sm text-text-muted mb-4">
          {{ description || '选择或创建一个 Git 仓库' }}
        </p>

        <div
          class="p-4 border border-border-normal rounded-lg cursor-pointer hover:shadow-md hover:border-brand/50 transition-all bg-bg-secondary"
          @click="stage = 'existing'"
        >
          <div class="flex items-start gap-3">
            <svg class="w-5 h-5 mt-0.5 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <div class="min-w-0 flex-1">
              <div class="font-medium text-text-primary">从 Git 仓库</div>
              <div class="text-xs text-text-muted mt-1">
                选择系统上已有的 Git 仓库
              </div>
            </div>
          </div>
        </div>

        <div
          class="p-4 border border-border-normal rounded-lg cursor-pointer hover:shadow-md hover:border-brand/50 transition-all bg-bg-secondary"
          @click="stage = 'new'"
        >
          <div class="flex items-start gap-3">
            <svg class="w-5 h-5 mt-0.5 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <div class="min-w-0 flex-1">
              <div class="font-medium text-text-primary">创建新仓库</div>
              <div class="text-xs text-text-muted mt-1">
                初始化一个新的 Git 仓库
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- Stage: Existing -->
      <template v-else-if="stage === 'existing'">
        <button
          class="text-sm text-text-muted hover:text-text-primary flex items-center gap-1"
          :disabled="isWorking"
          @click="goBack"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          返回选项
        </button>

        <!-- Loading -->
        <div v-if="reposLoading" class="p-4 border border-border-normal rounded-lg bg-bg-secondary">
          <div class="flex items-center gap-3">
            <Loading size="sm" />
            <div class="text-sm text-text-muted">
              {{ loadingDuration < 2 ? '搜索仓库中...' : `搜索中... (${loadingDuration}秒)` }}
            </div>
          </div>
          <div v-if="loadingDuration >= 3" class="text-xs text-text-muted mt-2 ml-8">
            正在扫描您的文件系统，这可能需要一些时间...
          </div>
        </div>

        <!-- Repo List -->
        <div v-else-if="allRepos.length > 0" class="space-y-2">
          <div
            v-for="repo in displayedRepos"
            :key="repo.path"
            class="p-4 border border-border-normal rounded-lg cursor-pointer hover:shadow-md hover:border-brand/50 transition-all bg-bg-secondary"
            @click="handleSelectRepo(repo)"
          >
            <div class="flex items-start gap-3">
              <svg class="w-5 h-5 mt-0.5 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <div class="min-w-0 flex-1">
                <div class="font-medium text-text-primary">{{ repo.name }}</div>
                <div class="text-xs text-text-muted truncate mt-1">{{ repo.path }}</div>
              </div>
            </div>
          </div>

          <button
            v-if="!showMoreRepos && allRepos.length > 3"
            class="text-sm text-text-muted hover:text-text-primary transition-colors"
            @click="showMoreRepos = true"
          >
            显示更多 ({{ allRepos.length - 3 }} 个)
          </button>
          <button
            v-else-if="showMoreRepos && allRepos.length > 3"
            class="text-sm text-text-muted hover:text-text-primary transition-colors"
            @click="showMoreRepos = false"
          >
            收起
          </button>
        </div>

        <!-- No repos found -->
        <div v-else-if="hasSearched && !error" class="p-4 border border-border-normal rounded-lg bg-bg-secondary">
          <div class="flex items-start gap-3">
            <svg class="w-5 h-5 mt-0.5 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <div>
              <div class="text-sm text-text-muted">未找到 Git 仓库</div>
              <div class="text-xs text-text-muted mt-1">可以手动输入仓库路径</div>
            </div>
          </div>
        </div>

        <!-- Manual path input -->
        <div class="pt-2 border-t border-border-normal">
          <label class="block text-sm font-medium text-text-secondary mb-1">
            手动输入仓库路径
          </label>
          <div class="flex gap-2">
            <Input
              v-model="parentPath"
              placeholder="/path/to/your/repo"
              :disabled="isWorking"
              class="flex-1"
            />
            <Button
              variant="secondary"
              :disabled="!parentPath.trim() || isWorking"
              :loading="isWorking"
              @click="registerAndReturn(parentPath.trim())"
            >
              选择
            </Button>
          </div>
        </div>
      </template>

      <!-- Stage: New -->
      <template v-else-if="stage === 'new'">
        <button
          class="text-sm text-text-muted hover:text-text-primary flex items-center gap-1"
          :disabled="isWorking"
          @click="goBack"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          返回选项
        </button>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-1">
              仓库名称 <span class="text-red-500">*</span>
            </label>
            <Input
              v-model="repoName"
              placeholder="my-project"
              :disabled="isWorking"
            />
            <p class="text-xs text-text-muted mt-1">
              这将作为新仓库的文件夹名称
            </p>
          </div>

          <div>
            <label class="block text-sm font-medium text-text-secondary mb-1">
              父目录
            </label>
            <Input
              v-model="parentPath"
              placeholder="当前工作目录"
              :disabled="isWorking"
            />
            <p class="text-xs text-text-muted mt-1">
              留空则使用当前工作目录
            </p>
          </div>

          <Button
            variant="primary"
            class="w-full"
            :disabled="!repoName.trim() || isWorking"
            :loading="isWorking"
            @click="handleCreateRepo"
          >
            创建仓库
          </Button>
        </div>
      </template>

      <!-- Error -->
      <div v-if="error" class="p-3 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm rounded-lg">
        {{ error }}
      </div>

      <!-- Working indicator -->
      <div v-if="isWorking && stage === 'existing'" class="flex items-center justify-center gap-2 text-sm text-text-muted">
        <Loading size="sm" />
        注册仓库中...
      </div>
    </div>

    <template #footer>
      <Button variant="secondary" :disabled="isWorking" @click="handleClose">
        取消
      </Button>
    </template>
  </Dialog>
</template>
