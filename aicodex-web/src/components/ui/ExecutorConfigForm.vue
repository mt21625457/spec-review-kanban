<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Input, Select } from '@/components/ui'
import { Info, AlertCircle } from 'lucide-vue-next'
import type { ExecutorConfig } from '@/types/settings'

// 字段定义类型
export interface FieldDefinition {
  key: string
  label: string
  type: 'text' | 'number' | 'select' | 'checkbox' | 'textarea' | 'password'
  placeholder?: string
  description?: string
  required?: boolean
  options?: Array<{ value: string | number; label: string }>
  min?: number
  max?: number
  step?: number
  rows?: number
  condition?: (config: Record<string, unknown>) => boolean
  validate?: (value: unknown) => string | null
}

// 执行器类型的字段配置
const executorFieldDefinitions: Record<string, FieldDefinition[]> = {
  'claude-code': [
    {
      key: 'api_key',
      label: 'API Key',
      type: 'password',
      placeholder: 'sk-ant-...',
      description: 'Anthropic API 密钥',
      required: true,
    },
    {
      key: 'model',
      label: '模型',
      type: 'select',
      options: [
        { value: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4' },
        { value: 'claude-3-5-sonnet-20241022', label: 'Claude 3.5 Sonnet' },
        { value: 'claude-3-opus-20240229', label: 'Claude 3 Opus' },
        { value: 'claude-3-haiku-20240307', label: 'Claude 3 Haiku' },
      ],
      description: '使用的 Claude 模型',
    },
    {
      key: 'max_tokens',
      label: '最大 Token 数',
      type: 'number',
      placeholder: '4096',
      min: 1,
      max: 200000,
      description: '单次响应的最大 token 数量',
    },
    {
      key: 'dangerously_skip_permissions',
      label: '跳过权限检查',
      type: 'checkbox',
      description: '危险：跳过所有权限确认提示',
    },
  ],
  'aider': [
    {
      key: 'api_key',
      label: 'API Key',
      type: 'password',
      placeholder: 'sk-...',
      description: 'OpenAI 或兼容 API 的密钥',
      required: true,
    },
    {
      key: 'model',
      label: '模型',
      type: 'text',
      placeholder: 'gpt-4o',
      description: '使用的模型名称',
    },
    {
      key: 'api_base',
      label: 'API Base URL',
      type: 'text',
      placeholder: 'https://api.openai.com/v1',
      description: '自定义 API 基础地址',
    },
    {
      key: 'auto_commits',
      label: '自动提交',
      type: 'checkbox',
      description: '自动提交代码更改',
    },
    {
      key: 'dirty_commits',
      label: '允许脏提交',
      type: 'checkbox',
      description: '即使有未暂存的更改也允许提交',
    },
  ],
  'codex': [
    {
      key: 'api_key',
      label: 'API Key',
      type: 'password',
      placeholder: 'sk-...',
      description: 'OpenAI API 密钥',
      required: true,
    },
    {
      key: 'model',
      label: '模型',
      type: 'select',
      options: [
        { value: 'o3', label: 'o3' },
        { value: 'o4-mini', label: 'o4-mini' },
        { value: 'gpt-4.1', label: 'GPT-4.1' },
      ],
      description: '使用的 Codex 模型',
    },
    {
      key: 'writable_root',
      label: '可写根目录',
      type: 'text',
      placeholder: '/path/to/project',
      description: '允许写入的根目录路径',
    },
  ],
  'custom': [
    {
      key: 'command',
      label: '执行命令',
      type: 'text',
      placeholder: '/path/to/executable',
      description: '自定义执行器的命令路径',
      required: true,
    },
    {
      key: 'args',
      label: '命令参数',
      type: 'textarea',
      placeholder: '--flag1\n--flag2=value',
      description: '每行一个参数',
      rows: 3,
    },
    {
      key: 'env',
      label: '环境变量',
      type: 'textarea',
      placeholder: 'KEY1=value1\nKEY2=value2',
      description: '每行一个环境变量',
      rows: 3,
    },
    {
      key: 'working_dir',
      label: '工作目录',
      type: 'text',
      placeholder: '${REPO_ROOT}',
      description: '执行命令的工作目录',
    },
  ],
}

const props = defineProps<{
  executorType: string
  modelValue: ExecutorConfig
}>()

const emit = defineEmits<{
  'update:modelValue': [config: ExecutorConfig]
}>()

const { t } = useI18n()

// 获取当前执行器类型的字段定义
const fields = computed(() => {
  return executorFieldDefinitions[props.executorType] || executorFieldDefinitions['custom']
})

// 获取字段值
const getFieldValue = (key: string): unknown => {
  return (props.modelValue as Record<string, unknown>)[key]
}

// 更新字段值
const updateField = (key: string, value: unknown) => {
  emit('update:modelValue', {
    ...props.modelValue,
    [key]: value,
  })
}

// 检查字段是否应该显示
const shouldShowField = (field: FieldDefinition): boolean => {
  if (!field.condition) return true
  return field.condition(props.modelValue as Record<string, unknown>)
}

// 验证字段
const validateField = (field: FieldDefinition): string | null => {
  const value = getFieldValue(field.key)

  if (field.required && (value === undefined || value === null || value === '')) {
    return t('common.required') || '此字段为必填项'
  }

  if (field.validate) {
    return field.validate(value)
  }

  return null
}
</script>

<template>
  <div class="space-y-4">
    <template v-for="field in fields" :key="field.key">
      <div v-if="shouldShowField(field)" class="space-y-1">
        <!-- 标签 -->
        <label class="flex items-center gap-2 text-sm font-medium text-text-primary">
          {{ field.label }}
          <span v-if="field.required" class="text-red-500">*</span>
          <span
            v-if="field.description"
            class="text-text-low cursor-help"
            :title="field.description"
          >
            <Info class="w-3.5 h-3.5" />
          </span>
        </label>

        <!-- 文本输入 -->
        <Input
          v-if="field.type === 'text'"
          :model-value="(getFieldValue(field.key) as string) || ''"
          :placeholder="field.placeholder"
          @update:model-value="updateField(field.key, $event)"
        />

        <!-- 密码输入 -->
        <Input
          v-else-if="field.type === 'password'"
          type="password"
          :model-value="(getFieldValue(field.key) as string) || ''"
          :placeholder="field.placeholder"
          @update:model-value="updateField(field.key, $event)"
        />

        <!-- 数字输入 -->
        <Input
          v-else-if="field.type === 'number'"
          type="number"
          :model-value="(getFieldValue(field.key) as number)?.toString() || ''"
          :placeholder="field.placeholder"
          :min="field.min"
          :max="field.max"
          :step="field.step"
          @update:model-value="updateField(field.key, $event ? Number($event) : undefined)"
        />

        <!-- 下拉选择 -->
        <Select
          v-else-if="field.type === 'select'"
          :model-value="(getFieldValue(field.key) as string | number) || null"
          :options="field.options || []"
          :placeholder="field.placeholder || t('common.select') || '请选择'"
          @update:model-value="updateField(field.key, $event)"
        />

        <!-- 复选框 -->
        <label
          v-else-if="field.type === 'checkbox'"
          class="flex items-center gap-2 cursor-pointer"
        >
          <input
            type="checkbox"
            :checked="(getFieldValue(field.key) as boolean) || false"
            class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            @change="updateField(field.key, ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-text-normal">
            {{ field.description }}
          </span>
        </label>

        <!-- 多行文本 -->
        <textarea
          v-else-if="field.type === 'textarea'"
          :value="(getFieldValue(field.key) as string) || ''"
          :placeholder="field.placeholder"
          :rows="field.rows || 3"
          class="input font-mono text-sm"
          @input="updateField(field.key, ($event.target as HTMLTextAreaElement).value)"
        />

        <!-- 描述文本 (非 checkbox 时显示) -->
        <p
          v-if="field.description && field.type !== 'checkbox'"
          class="text-xs text-text-low"
        >
          {{ field.description }}
        </p>

        <!-- 验证错误 -->
        <p
          v-if="validateField(field)"
          class="flex items-center gap-1 text-xs text-red-500"
        >
          <AlertCircle class="w-3.5 h-3.5" />
          {{ validateField(field) }}
        </p>
      </div>
    </template>

    <!-- 空状态 -->
    <div
      v-if="fields.length === 0"
      class="py-8 text-center text-text-low"
    >
      <p class="text-sm">{{ t('settings.agent.noConfigFields') || '此执行器类型暂无可配置字段' }}</p>
    </div>
  </div>
</template>
