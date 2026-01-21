<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { AlertCircle, Check, Copy, FileJson, Maximize2, Minimize2 } from 'lucide-vue-next'

const props = withDefaults(defineProps<{
  modelValue: string
  placeholder?: string
  readonly?: boolean
  minHeight?: string
  maxHeight?: string
  showToolbar?: boolean
  showLineNumbers?: boolean
}>(), {
  placeholder: '{}',
  readonly: false,
  minHeight: '200px',
  maxHeight: '500px',
  showToolbar: true,
  showLineNumbers: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'valid': [isValid: boolean]
}>()

const { t } = useI18n()

// 编辑器状态
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const isExpanded = ref(false)
const copied = ref(false)

// JSON 验证
const validationResult = computed(() => {
  if (!props.modelValue || props.modelValue.trim() === '') {
    return { valid: true, error: null }
  }
  try {
    JSON.parse(props.modelValue)
    return { valid: true, error: null }
  } catch (e) {
    const error = e as SyntaxError
    return { valid: false, error: error.message }
  }
})

// 行号计算
const lineCount = computed(() => {
  if (!props.modelValue) return 1
  return props.modelValue.split('\n').length
})

// 监听验证结果变化
watch(() => validationResult.value.valid, (isValid) => {
  emit('valid', isValid)
}, { immediate: true })

// 更新内容
const handleInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}

// 格式化 JSON
const formatJson = () => {
  if (!props.modelValue || props.readonly) return
  try {
    const parsed = JSON.parse(props.modelValue)
    const formatted = JSON.stringify(parsed, null, 2)
    emit('update:modelValue', formatted)
  } catch {
    // 如果解析失败，保持原样
  }
}

// 压缩 JSON
const minifyJson = () => {
  if (!props.modelValue || props.readonly) return
  try {
    const parsed = JSON.parse(props.modelValue)
    const minified = JSON.stringify(parsed)
    emit('update:modelValue', minified)
  } catch {
    // 如果解析失败，保持原样
  }
}

// 复制到剪贴板
const copyToClipboard = async () => {
  if (!props.modelValue) return
  try {
    await navigator.clipboard.writeText(props.modelValue)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    // 复制失败
  }
}

// 切换展开状态
const toggleExpand = () => {
  isExpanded.value = !isExpanded.value
}

// Tab 键插入空格
const handleKeydown = (event: KeyboardEvent) => {
  if (props.readonly) return

  if (event.key === 'Tab') {
    event.preventDefault()
    const textarea = textareaRef.value
    if (!textarea) return

    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    const value = props.modelValue

    const newValue = value.substring(0, start) + '  ' + value.substring(end)
    emit('update:modelValue', newValue)

    // 恢复光标位置
    requestAnimationFrame(() => {
      textarea.selectionStart = textarea.selectionEnd = start + 2
    })
  }
}

// 同步滚动（行号和编辑器）
const lineNumbersRef = ref<HTMLDivElement | null>(null)
const handleScroll = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  if (lineNumbersRef.value) {
    lineNumbersRef.value.scrollTop = target.scrollTop
  }
}

onMounted(() => {
  // 初始验证
  emit('valid', validationResult.value.valid)
})
</script>

<template>
  <div
    class="json-editor border border-border-normal rounded-lg overflow-hidden"
    :class="{ 'fixed inset-4 z-50 bg-bg-primary': isExpanded }"
  >
    <!-- 工具栏 -->
    <div
      v-if="showToolbar"
      class="flex items-center justify-between px-3 py-2 bg-bg-secondary border-b border-border-normal"
    >
      <div class="flex items-center gap-2">
        <FileJson class="w-4 h-4 text-text-low" />
        <span class="text-xs font-medium text-text-low">JSON</span>

        <!-- 验证状态 -->
        <div
          v-if="validationResult.valid"
          class="flex items-center gap-1 text-green-600 dark:text-green-400"
        >
          <Check class="w-3.5 h-3.5" />
          <span class="text-xs">{{ t('common.valid') || '有效' }}</span>
        </div>
        <div
          v-else
          class="flex items-center gap-1 text-red-500"
          :title="validationResult.error || ''"
        >
          <AlertCircle class="w-3.5 h-3.5" />
          <span class="text-xs truncate max-w-[200px]">{{ validationResult.error }}</span>
        </div>
      </div>

      <div class="flex items-center gap-1">
        <!-- 格式化按钮 -->
        <button
          v-if="!readonly"
          type="button"
          class="px-2 py-1 text-xs text-text-low hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
          :disabled="!validationResult.valid"
          @click="formatJson"
        >
          {{ t('common.format') || '格式化' }}
        </button>

        <!-- 压缩按钮 -->
        <button
          v-if="!readonly"
          type="button"
          class="px-2 py-1 text-xs text-text-low hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
          :disabled="!validationResult.valid"
          @click="minifyJson"
        >
          {{ t('common.minify') || '压缩' }}
        </button>

        <!-- 复制按钮 -->
        <button
          type="button"
          class="p-1 text-text-low hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
          @click="copyToClipboard"
        >
          <Check v-if="copied" class="w-4 h-4 text-green-500" />
          <Copy v-else class="w-4 h-4" />
        </button>

        <!-- 展开/收起按钮 -->
        <button
          type="button"
          class="p-1 text-text-low hover:text-text-primary hover:bg-bg-hover rounded transition-colors"
          @click="toggleExpand"
        >
          <Minimize2 v-if="isExpanded" class="w-4 h-4" />
          <Maximize2 v-else class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- 编辑区域 -->
    <div
      class="flex overflow-hidden"
      :style="{
        minHeight: isExpanded ? 'calc(100vh - 120px)' : minHeight,
        maxHeight: isExpanded ? 'calc(100vh - 120px)' : maxHeight
      }"
    >
      <!-- 行号 -->
      <div
        v-if="showLineNumbers"
        ref="lineNumbersRef"
        class="flex-shrink-0 py-3 px-2 bg-bg-secondary text-right text-xs font-mono text-text-low select-none overflow-hidden border-r border-border-normal"
        style="width: 40px;"
      >
        <div v-for="n in lineCount" :key="n" class="leading-5">
          {{ n }}
        </div>
      </div>

      <!-- 文本编辑器 -->
      <textarea
        ref="textareaRef"
        :value="modelValue"
        :placeholder="placeholder"
        :readonly="readonly"
        class="flex-1 p-3 font-mono text-sm leading-5 bg-bg-primary text-text-primary resize-none focus:outline-none"
        :class="{
          'cursor-not-allowed opacity-75': readonly,
          'border-red-500': !validationResult.valid
        }"
        spellcheck="false"
        @input="handleInput"
        @keydown="handleKeydown"
        @scroll="handleScroll"
      />
    </div>

    <!-- 错误详情（展开时显示） -->
    <div
      v-if="!validationResult.valid && validationResult.error"
      class="px-3 py-2 bg-red-50 dark:bg-red-900/20 border-t border-red-200 dark:border-red-800 text-sm text-red-600 dark:text-red-400"
    >
      {{ validationResult.error }}
    </div>
  </div>
</template>

<style scoped>
.json-editor textarea {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  tab-size: 2;
}

.json-editor textarea::placeholder {
  color: var(--color-text-low);
}
</style>
