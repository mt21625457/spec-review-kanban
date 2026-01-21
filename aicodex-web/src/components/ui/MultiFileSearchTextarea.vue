<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { File, FolderOpen, X, Plus, Search, AlertCircle } from 'lucide-vue-next'

const props = withDefaults(defineProps<{
  modelValue: string[]
  placeholder?: string
  maxItems?: number
  allowGlob?: boolean
  validatePath?: (path: string) => string | null
}>(), {
  placeholder: '',
  maxItems: 100,
  allowGlob: true,
})

const emit = defineEmits<{
  'update:modelValue': [paths: string[]]
}>()

const { t } = useI18n()

// 状态
const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const errors = ref<Map<number, string>>(new Map())

// 计算属性
const canAddMore = computed(() => props.modelValue.length < props.maxItems)

const isGlobPattern = (path: string): boolean => {
  return /[*?[\]{}]/.test(path)
}

// 添加路径
const addPath = () => {
  const trimmedValue = inputValue.value.trim()
  if (!trimmedValue) return

  // 验证
  if (props.validatePath) {
    const error = props.validatePath(trimmedValue)
    if (error) {
      // 显示错误但不添加
      return
    }
  }

  // 检查重复
  if (props.modelValue.includes(trimmedValue)) {
    return
  }

  // 添加
  emit('update:modelValue', [...props.modelValue, trimmedValue])
  inputValue.value = ''
  inputRef.value?.focus()
}

// 删除路径
const removePath = (index: number) => {
  const newPaths = [...props.modelValue]
  newPaths.splice(index, 1)
  emit('update:modelValue', newPaths)
  errors.value.delete(index)
}

// 更新路径
const updatePath = (index: number, value: string) => {
  const newPaths = [...props.modelValue]
  newPaths[index] = value
  emit('update:modelValue', newPaths)

  // 验证
  if (props.validatePath) {
    const error = props.validatePath(value)
    if (error) {
      errors.value.set(index, error)
    } else {
      errors.value.delete(index)
    }
  }
}

// 处理键盘事件
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    event.preventDefault()
    addPath()
  }
}

// 从文本批量导入
const bulkImport = (text: string) => {
  const lines = text.split('\n').map(line => line.trim()).filter(line => line)
  const newPaths = [...props.modelValue]

  for (const line of lines) {
    if (!newPaths.includes(line) && newPaths.length < props.maxItems) {
      newPaths.push(line)
    }
  }

  emit('update:modelValue', newPaths)
}

// 处理粘贴事件
const handlePaste = (event: ClipboardEvent) => {
  const text = event.clipboardData?.getData('text')
  if (text && text.includes('\n')) {
    event.preventDefault()
    bulkImport(text)
  }
}

// 导出为文本
const exportAsText = computed(() => props.modelValue.join('\n'))

// 复制所有内容
const copyAll = async () => {
  try {
    await navigator.clipboard.writeText(exportAsText.value)
  } catch {
    // 复制失败
  }
}
</script>

<template>
  <div class="space-y-3">
    <!-- 输入区域 -->
    <div class="flex gap-2">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-low" />
        <input
          ref="inputRef"
          v-model="inputValue"
          type="text"
          class="input pl-9 pr-3 py-2 w-full"
          :placeholder="placeholder || t('settings.multiFile.placeholder') || '输入文件路径或 glob 模式...'"
          :disabled="!canAddMore"
          @keydown="handleKeydown"
          @paste="handlePaste"
        />
      </div>
      <button
        type="button"
        class="btn btn-secondary px-3"
        :disabled="!inputValue.trim() || !canAddMore"
        @click="addPath"
      >
        <Plus class="w-4 h-4" />
      </button>
    </div>

    <!-- 提示信息 -->
    <div v-if="allowGlob" class="text-xs text-text-low">
      {{ t('settings.multiFile.globHint') || '支持 glob 模式，如 **/*.ts、src/**/*.vue' }}
    </div>

    <!-- 路径列表 -->
    <div
      v-if="modelValue.length > 0"
      class="space-y-2 max-h-[300px] overflow-y-auto"
    >
      <div
        v-for="(path, index) in modelValue"
        :key="index"
        class="group flex items-center gap-2 p-2 bg-bg-secondary rounded-lg border border-border-normal hover:border-border-hover transition-colors"
        :class="{ 'border-red-300 dark:border-red-700': errors.has(index) }"
      >
        <!-- 图标 -->
        <component
          :is="isGlobPattern(path) ? Search : path.endsWith('/') ? FolderOpen : File"
          class="w-4 h-4 flex-shrink-0"
          :class="isGlobPattern(path) ? 'text-brand' : 'text-text-low'"
        />

        <!-- 可编辑路径 -->
        <input
          :value="path"
          type="text"
          class="flex-1 bg-transparent text-sm font-mono text-text-primary focus:outline-none"
          @input="updatePath(index, ($event.target as HTMLInputElement).value)"
        />

        <!-- 错误提示 -->
        <div
          v-if="errors.has(index)"
          class="flex items-center gap-1 text-xs text-red-500"
          :title="errors.get(index)"
        >
          <AlertCircle class="w-3.5 h-3.5" />
        </div>

        <!-- 删除按钮 -->
        <button
          type="button"
          class="p-1 text-text-low hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded opacity-0 group-hover:opacity-100 transition-all"
          @click="removePath(index)"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-else
      class="py-8 text-center text-text-low"
    >
      <FolderOpen class="w-8 h-8 mx-auto mb-2 opacity-50" />
      <p class="text-sm">{{ t('settings.multiFile.empty') || '暂无文件路径' }}</p>
      <p class="text-xs mt-1">{{ t('settings.multiFile.emptyHint') || '输入路径或粘贴多行内容以批量添加' }}</p>
    </div>

    <!-- 底部信息 -->
    <div class="flex items-center justify-between text-xs text-text-low">
      <span>{{ modelValue.length }} / {{ maxItems }} {{ t('settings.multiFile.count') || '个路径' }}</span>
      <button
        v-if="modelValue.length > 0"
        type="button"
        class="text-brand hover:underline"
        @click="copyAll"
      >
        {{ t('common.copyAll') || '复制全部' }}
      </button>
    </div>
  </div>
</template>
