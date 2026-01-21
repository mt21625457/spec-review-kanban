<script setup lang="ts">
/**
 * DiffViewer - Component for displaying file diffs
 */
import { computed } from 'vue'
import type { Diff, DiffChangeKind } from '@/types'

const props = defineProps<{
  diff: Diff
  expanded?: boolean
}>()

const emit = defineEmits<{
  toggle: []
}>()

const changeInfo: Record<DiffChangeKind, { label: string; color: string; bgColor: string }> = {
  added: {
    label: '新增',
    color: 'text-green-600 dark:text-green-400',
    bgColor: 'bg-green-50 dark:bg-green-900/20',
  },
  deleted: {
    label: '删除',
    color: 'text-red-600 dark:text-red-400',
    bgColor: 'bg-red-50 dark:bg-red-900/20',
  },
  modified: {
    label: '修改',
    color: 'text-yellow-600 dark:text-yellow-400',
    bgColor: 'bg-yellow-50 dark:bg-yellow-900/20',
  },
  renamed: {
    label: '重命名',
    color: 'text-blue-600 dark:text-blue-400',
    bgColor: 'bg-blue-50 dark:bg-blue-900/20',
  },
  copied: {
    label: '复制',
    color: 'text-purple-600 dark:text-purple-400',
    bgColor: 'bg-purple-50 dark:bg-purple-900/20',
  },
  permissionChange: {
    label: '权限变更',
    color: 'text-gray-600 dark:text-gray-400',
    bgColor: 'bg-gray-50 dark:bg-gray-900/20',
  },
}

const info = computed(() => changeInfo[props.diff.change])

const displayPath = computed(() => {
  if (props.diff.change === 'renamed' && props.diff.oldPath && props.diff.newPath) {
    return `${props.diff.oldPath} → ${props.diff.newPath}`
  }
  return props.diff.newPath || props.diff.oldPath || 'Unknown'
})

const fileName = computed(() => {
  const path = props.diff.newPath || props.diff.oldPath || ''
  return path.split('/').pop() || path
})

// Simple diff line parser
const diffLines = computed(() => {
  const lines: Array<{ type: 'add' | 'remove' | 'context'; content: string; lineNumber?: number }> = []

  if (props.diff.change === 'added' && props.diff.newContent) {
    props.diff.newContent.split('\n').forEach((line, i) => {
      lines.push({ type: 'add', content: line, lineNumber: i + 1 })
    })
  } else if (props.diff.change === 'deleted' && props.diff.oldContent) {
    props.diff.oldContent.split('\n').forEach((line, i) => {
      lines.push({ type: 'remove', content: line, lineNumber: i + 1 })
    })
  } else if (props.diff.oldContent && props.diff.newContent) {
    // Simple unified diff display
    const oldLines = props.diff.oldContent.split('\n')
    const newLines = props.diff.newContent.split('\n')

    // For simplicity, show removed then added
    oldLines.forEach((line, i) => {
      if (!newLines.includes(line)) {
        lines.push({ type: 'remove', content: line, lineNumber: i + 1 })
      }
    })
    newLines.forEach((line, i) => {
      if (!oldLines.includes(line)) {
        lines.push({ type: 'add', content: line, lineNumber: i + 1 })
      }
    })
  }

  return lines
})

const hasContent = computed(() => diffLines.value.length > 0)

const handleToggle = () => {
  emit('toggle')
}
</script>

<template>
  <div class="border border-border-normal rounded-lg overflow-hidden">
    <!-- Header -->
    <div
      class="flex items-center justify-between px-3 py-2 cursor-pointer hover:bg-bg-hover transition-colors"
      :class="info.bgColor"
      @click="handleToggle"
    >
      <div class="flex items-center gap-2 min-w-0">
        <!-- Expand/Collapse Icon -->
        <svg
          class="w-4 h-4 text-text-muted transition-transform flex-shrink-0"
          :class="{ 'rotate-90': expanded }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>

        <!-- File Icon -->
        <svg class="w-4 h-4 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>

        <!-- File Name -->
        <span class="text-sm font-medium text-text-primary truncate" :title="displayPath">
          {{ fileName }}
        </span>

        <!-- Full Path (on hover) -->
        <span v-if="displayPath !== fileName" class="text-xs text-text-muted truncate hidden sm:inline">
          {{ displayPath }}
        </span>
      </div>

      <!-- Change Badge -->
      <span
        class="text-xs font-medium px-2 py-0.5 rounded flex-shrink-0"
        :class="info.color"
      >
        {{ info.label }}
      </span>
    </div>

    <!-- Content -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="max-h-0 opacity-0"
      enter-to-class="max-h-[500px] opacity-100"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="max-h-[500px] opacity-100"
      leave-to-class="max-h-0 opacity-0"
    >
      <div v-if="expanded && hasContent" class="overflow-hidden">
        <div class="max-h-96 overflow-y-auto bg-gray-900 font-mono text-xs">
          <table class="w-full">
            <tbody>
              <tr
                v-for="(line, index) in diffLines"
                :key="index"
                :class="{
                  'bg-green-900/30': line.type === 'add',
                  'bg-red-900/30': line.type === 'remove',
                }"
              >
                <!-- Line Number -->
                <td class="px-2 py-0.5 text-right text-gray-500 select-none w-12 border-r border-gray-700">
                  {{ line.lineNumber }}
                </td>
                <!-- Change Indicator -->
                <td class="px-1 py-0.5 text-center select-none w-6">
                  <span
                    :class="{
                      'text-green-400': line.type === 'add',
                      'text-red-400': line.type === 'remove',
                    }"
                  >
                    {{ line.type === 'add' ? '+' : line.type === 'remove' ? '-' : ' ' }}
                  </span>
                </td>
                <!-- Content -->
                <td class="px-2 py-0.5 whitespace-pre-wrap break-all">
                  <span
                    :class="{
                      'text-green-300': line.type === 'add',
                      'text-red-300': line.type === 'remove',
                      'text-gray-300': line.type === 'context',
                    }"
                  >{{ line.content || ' ' }}</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </Transition>

    <!-- Empty State -->
    <div v-if="expanded && !hasContent" class="px-4 py-6 text-center text-text-muted text-sm bg-bg-secondary">
      暂无变更内容
    </div>
  </div>
</template>
