<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  minRows?: number
  maxRows?: number
  disabled?: boolean
}>(), {
  modelValue: '',
  minRows: 2,
  maxRows: 10,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const textareaRef = ref<HTMLTextAreaElement | null>(null)

const adjustHeight = () => {
  const textarea = textareaRef.value
  if (!textarea) return

  // 重置高度以获取正确的 scrollHeight
  textarea.style.height = 'auto'

  // 计算行高
  const lineHeight = parseInt(getComputedStyle(textarea).lineHeight) || 20
  const minHeight = props.minRows * lineHeight
  const maxHeight = props.maxRows * lineHeight

  // 设置新高度
  const newHeight = Math.min(Math.max(textarea.scrollHeight, minHeight), maxHeight)
  textarea.style.height = `${newHeight}px`
}

const handleInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
  nextTick(adjustHeight)
}

watch(() => props.modelValue, () => {
  nextTick(adjustHeight)
})

onMounted(() => {
  nextTick(adjustHeight)
})
</script>

<template>
  <textarea
    ref="textareaRef"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    class="input font-mono text-sm resize-none overflow-hidden"
    :class="{ 'opacity-50 cursor-not-allowed': disabled }"
    @input="handleInput"
  />
</template>
