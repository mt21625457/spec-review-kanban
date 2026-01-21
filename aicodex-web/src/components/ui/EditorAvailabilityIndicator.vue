<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Check, X, Loader2 } from 'lucide-vue-next'

const props = defineProps<{
  editorType: string
  customCommand?: string
}>()

const { t } = useI18n()

// 可用性状态
const checking = ref(false)
const available = ref<boolean | null>(null)
const message = ref<string>('')

// 编辑器命令映射
const editorCommands: Record<string, string> = {
  vscode: 'code',
  cursor: 'cursor',
  windsurf: 'windsurf',
  zed: 'zed',
}

// 获取检查的命令
const commandToCheck = computed(() => {
  if (props.editorType === 'custom') {
    return props.customCommand?.split(' ')[0] || ''
  }
  return editorCommands[props.editorType] || ''
})

// 检查编辑器可用性（本地检查）
const checkAvailability = async () => {
  const cmd = commandToCheck.value
  if (!cmd) {
    available.value = null
    return
  }

  checking.value = true
  available.value = null

  try {
    // 调用后端 API 检查编辑器可用性
    const response = await fetch(`/api/proxy/editors/check-availability?editor_type=${encodeURIComponent(props.editorType)}`)
    if (response.ok) {
      const result = await response.json()
      available.value = result.data?.available ?? result.available ?? false
      message.value = result.data?.message ?? result.message ?? ''
    } else {
      // 如果 API 不存在，假设可用
      available.value = true
      message.value = ''
    }
  } catch {
    // API 调用失败，假设可用
    available.value = true
    message.value = ''
  } finally {
    checking.value = false
  }
}

// 监听编辑器类型和自定义命令变化
watch([() => props.editorType, () => props.customCommand], () => {
  checkAvailability()
}, { immediate: true })
</script>

<template>
  <div class="flex items-center gap-1.5">
    <template v-if="checking">
      <Loader2 class="w-4 h-4 animate-spin text-text-low" />
      <span class="text-sm text-text-low">
        {{ t('settings.general.editor.checking') }}
      </span>
    </template>
    <template v-else-if="available !== null">
      <template v-if="available">
        <Check class="w-4 h-4 text-green-500" />
        <span class="text-sm text-green-600 dark:text-green-400">
          {{ t('settings.general.editor.available') }}
        </span>
      </template>
      <template v-else>
        <X class="w-4 h-4 text-red-500" />
        <span class="text-sm text-red-600 dark:text-red-400">
          {{ message || t('settings.general.editor.unavailable') }}
        </span>
      </template>
    </template>
  </div>
</template>
