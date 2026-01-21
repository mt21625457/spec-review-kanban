<script setup lang="ts">
/**
 * DiffsPanel - Panel for viewing file diffs in a workspace
 */
import { computed, ref, toRef, watch } from 'vue'
import { useUiStore } from '@/stores'
import { useWorkspace } from '@/composables/useWorkspaces'
import DiffViewer from './DiffViewer.vue'
import Button from '@/components/ui/Button.vue'
import Loading from '@/components/ui/Loading.vue'
import type { Diff } from '@/types'

const uiStore = useUiStore()

// Get workspace ID from UI store
const workspaceId = toRef(uiStore, 'selectedAttemptId')
const { data: workspace, isLoading: workspaceLoading } = useWorkspace(workspaceId)

// Placeholder for diffs - in a real implementation, this would come from an API
const diffs = ref<Diff[]>([])
const diffsLoading = ref(false)
const diffsError = ref<Error | null>(null)

// Track expanded state for each diff
const expandedDiffs = ref<Set<number>>(new Set())

// Computed
const isOpen = computed(() => uiStore.activePanelType === 'diffs' && !!workspaceId.value)
const isLoading = computed(() => workspaceLoading.value || diffsLoading.value)

const totalChanges = computed(() => diffs.value.length)

// Methods
const handleClose = () => {
  uiStore.closePanel()
}

const toggleDiff = (index: number) => {
  if (expandedDiffs.value.has(index)) {
    expandedDiffs.value.delete(index)
  } else {
    expandedDiffs.value.add(index)
  }
}

const expandAll = () => {
  diffs.value.forEach((_, index) => {
    expandedDiffs.value.add(index)
  })
}

const collapseAll = () => {
  expandedDiffs.value.clear()
}

const handleBackToWorkspace = () => {
  if (workspaceId.value) {
    uiStore.openAttemptPanel(workspaceId.value)
  }
}

// Mock loading some diffs for demonstration
// In real implementation, this would fetch from API
const loadDiffs = async () => {
  diffsLoading.value = true
  diffsError.value = null

  try {
    // Simulated delay
    await new Promise((resolve) => setTimeout(resolve, 500))

    // Mock data for demonstration
    diffs.value = [
      {
        change: 'modified',
        oldPath: 'src/App.vue',
        newPath: 'src/App.vue',
        oldContent: '<template>\n  <div>Old content</div>\n</template>',
        newContent: '<template>\n  <div>New content with changes</div>\n</template>',
      },
      {
        change: 'added',
        oldPath: null,
        newPath: 'src/components/NewComponent.vue',
        oldContent: null,
        newContent: '<template>\n  <div>New component</div>\n</template>\n\n<script setup lang="ts">\n// Component logic\n</' + 'script>',
      },
    ]
  } catch (e) {
    diffsError.value = e instanceof Error ? e : new Error('Failed to load diffs')
  } finally {
    diffsLoading.value = false
  }
}

// Load diffs when panel opens
watch(isOpen, (open) => {
  if (open) {
    loadDiffs()
  }
}, { immediate: true })
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
        class="fixed inset-y-0 right-0 w-full max-w-3xl bg-bg-primary border-l border-border-normal shadow-xl z-40 flex flex-col"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3 border-b border-border-normal">
          <div class="flex items-center gap-3">
            <button
              type="button"
              class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
              title="返回工作区"
              @click="handleBackToWorkspace"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <h2 class="text-lg font-semibold text-text-primary">文件变更</h2>
            <span v-if="totalChanges > 0" class="text-xs text-text-muted bg-bg-secondary px-1.5 py-0.5 rounded">
              {{ totalChanges }} 个文件
            </span>
          </div>

          <div class="flex items-center gap-2">
            <!-- Expand/Collapse All -->
            <button
              type="button"
              class="text-xs text-text-muted hover:text-text-primary"
              @click="expandAll"
            >
              展开全部
            </button>
            <span class="text-text-muted">|</span>
            <button
              type="button"
              class="text-xs text-text-muted hover:text-text-primary"
              @click="collapseAll"
            >
              收起全部
            </button>

            <!-- Close -->
            <button
              type="button"
              class="p-1 text-text-muted hover:text-text-primary hover:bg-bg-hover rounded transition-colors ml-2"
              @click="handleClose"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto p-4">
          <!-- Loading -->
          <div v-if="isLoading" class="flex items-center justify-center h-64">
            <Loading />
          </div>

          <!-- Error -->
          <div v-else-if="diffsError" class="text-center py-12">
            <div class="text-red-500 mb-4">
              <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <p class="text-text-secondary">{{ diffsError.message }}</p>
            </div>
            <Button variant="secondary" @click="loadDiffs">重试</Button>
          </div>

          <!-- Empty State -->
          <div v-else-if="diffs.length === 0" class="text-center py-12">
            <div class="text-text-muted mb-4">
              <svg class="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p>暂无文件变更</p>
            </div>
          </div>

          <!-- Diffs List -->
          <div v-else class="space-y-3">
            <DiffViewer
              v-for="(diff, index) in diffs"
              :key="index"
              :diff="diff"
              :expanded="expandedDiffs.has(index)"
              @toggle="toggleDiff(index)"
            />
          </div>
        </div>

        <!-- Footer -->
        <div v-if="workspace" class="px-4 py-3 border-t border-border-normal bg-bg-secondary">
          <div class="flex items-center justify-between text-sm">
            <div class="text-text-muted">
              分支: <span class="text-text-primary font-mono">{{ workspace.branch }}</span>
            </div>
            <Button variant="primary" size="sm" @click="handleBackToWorkspace">
              返回工作区
            </Button>
          </div>
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
