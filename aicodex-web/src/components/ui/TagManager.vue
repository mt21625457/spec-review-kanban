<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Input } from '@/components/ui'
import { Plus, X, Edit2, Check, AlertCircle } from 'lucide-vue-next'

export interface Tag {
  id: string
  name: string
  color?: string
}

const props = withDefaults(defineProps<{
  modelValue: Tag[]
  placeholder?: string
  maxTags?: number
  allowCreate?: boolean
  allowEdit?: boolean
  allowDelete?: boolean
  colorOptions?: string[]
}>(), {
  placeholder: '',
  maxTags: 50,
  allowCreate: true,
  allowEdit: true,
  allowDelete: true,
  colorOptions: () => [
    '#3B82F6', // blue
    '#10B981', // green
    '#F59E0B', // amber
    '#EF4444', // red
    '#8B5CF6', // violet
    '#EC4899', // pink
    '#06B6D4', // cyan
    '#F97316', // orange
    '#6366F1', // indigo
    '#84CC16', // lime
  ],
})

const emit = defineEmits<{
  'update:modelValue': [tags: Tag[]]
  'create': [tag: Omit<Tag, 'id'>]
  'update': [tag: Tag]
  'delete': [tagId: string]
}>()

const { t } = useI18n()

// 状态
const isCreating = ref(false)
const editingTagId = ref<string | null>(null)
const newTagName = ref('')
const newTagColor = ref(props.colorOptions[0])
const editTagName = ref('')
const editTagColor = ref('')
const error = ref('')

// 计算属性
const canCreateMore = computed(() => props.modelValue.length < props.maxTags)

// 验证标签名称
const validateTagName = (name: string, excludeId?: string): string | null => {
  const trimmedName = name.trim()
  if (!trimmedName) {
    return t('settings.tags.errorEmpty') || '标签名称不能为空'
  }
  if (trimmedName.length > 50) {
    return t('settings.tags.errorTooLong') || '标签名称不能超过50个字符'
  }
  const isDuplicate = props.modelValue.some(
    tag => tag.name.toLowerCase() === trimmedName.toLowerCase() && tag.id !== excludeId
  )
  if (isDuplicate) {
    return t('settings.tags.errorDuplicate') || '标签名称已存在'
  }
  return null
}

// 开始创建标签
const startCreate = () => {
  isCreating.value = true
  newTagName.value = ''
  newTagColor.value = props.colorOptions[Math.floor(Math.random() * props.colorOptions.length)]
  error.value = ''
}

// 取消创建
const cancelCreate = () => {
  isCreating.value = false
  newTagName.value = ''
  error.value = ''
}

// 确认创建
const confirmCreate = () => {
  const validationError = validateTagName(newTagName.value)
  if (validationError) {
    error.value = validationError
    return
  }

  const newTag: Omit<Tag, 'id'> = {
    name: newTagName.value.trim(),
    color: newTagColor.value,
  }

  emit('create', newTag)

  // 乐观更新：本地添加临时 ID
  const tempTag: Tag = {
    ...newTag,
    id: `temp-${Date.now()}`,
  }
  emit('update:modelValue', [...props.modelValue, tempTag])

  cancelCreate()
}

// 开始编辑标签
const startEdit = (tag: Tag) => {
  editingTagId.value = tag.id
  editTagName.value = tag.name
  editTagColor.value = tag.color || props.colorOptions[0]
  error.value = ''
}

// 取消编辑
const cancelEdit = () => {
  editingTagId.value = null
  editTagName.value = ''
  editTagColor.value = ''
  error.value = ''
}

// 确认编辑
const confirmEdit = () => {
  if (!editingTagId.value) return

  const validationError = validateTagName(editTagName.value, editingTagId.value)
  if (validationError) {
    error.value = validationError
    return
  }

  const updatedTag: Tag = {
    id: editingTagId.value,
    name: editTagName.value.trim(),
    color: editTagColor.value,
  }

  emit('update', updatedTag)

  // 乐观更新
  const updatedTags = props.modelValue.map(tag =>
    tag.id === editingTagId.value ? updatedTag : tag
  )
  emit('update:modelValue', updatedTags)

  cancelEdit()
}

// 删除标签
const deleteTag = (tagId: string) => {
  if (!confirm(t('settings.tags.confirmDelete') || '确定要删除此标签吗？')) {
    return
  }

  emit('delete', tagId)

  // 乐观更新
  const updatedTags = props.modelValue.filter(tag => tag.id !== tagId)
  emit('update:modelValue', updatedTags)
}

// 处理键盘事件
const handleKeydown = (event: KeyboardEvent, action: 'create' | 'edit') => {
  if (event.key === 'Enter') {
    event.preventDefault()
    if (action === 'create') {
      confirmCreate()
    } else {
      confirmEdit()
    }
  } else if (event.key === 'Escape') {
    if (action === 'create') {
      cancelCreate()
    } else {
      cancelEdit()
    }
  }
}
</script>

<template>
  <div class="space-y-3">
    <!-- 标签列表 -->
    <div class="flex flex-wrap gap-2">
      <template v-for="tag in modelValue" :key="tag.id">
        <!-- 编辑模式 -->
        <div
          v-if="editingTagId === tag.id"
          class="flex items-center gap-2 p-2 bg-bg-secondary rounded-lg border border-border-normal"
        >
          <Input
            v-model="editTagName"
            class="w-32 text-sm"
            :placeholder="t('settings.tags.namePlaceholder') || '标签名称'"
            @keydown="handleKeydown($event, 'edit')"
          />
          <div class="flex gap-1">
            <button
              v-for="color in colorOptions"
              :key="color"
              type="button"
              class="w-5 h-5 rounded-full border-2 transition-transform hover:scale-110"
              :class="{ 'border-text-primary scale-110': editTagColor === color, 'border-transparent': editTagColor !== color }"
              :style="{ backgroundColor: color }"
              @click="editTagColor = color"
            />
          </div>
          <button
            type="button"
            class="p-1 text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded"
            @click="confirmEdit"
          >
            <Check class="w-4 h-4" />
          </button>
          <button
            type="button"
            class="p-1 text-text-low hover:bg-bg-hover rounded"
            @click="cancelEdit"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- 显示模式 -->
        <div
          v-else
          class="group flex items-center gap-1.5 px-3 py-1.5 rounded-full text-sm font-medium transition-colors"
          :style="{
            backgroundColor: (tag.color || colorOptions[0]) + '20',
            color: tag.color || colorOptions[0],
          }"
        >
          <span
            class="w-2 h-2 rounded-full"
            :style="{ backgroundColor: tag.color || colorOptions[0] }"
          />
          <span>{{ tag.name }}</span>
          <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              v-if="allowEdit"
              type="button"
              class="p-0.5 hover:bg-white/30 dark:hover:bg-black/20 rounded"
              @click="startEdit(tag)"
            >
              <Edit2 class="w-3 h-3" />
            </button>
            <button
              v-if="allowDelete"
              type="button"
              class="p-0.5 hover:bg-white/30 dark:hover:bg-black/20 rounded"
              @click="deleteTag(tag.id)"
            >
              <X class="w-3 h-3" />
            </button>
          </div>
        </div>
      </template>

      <!-- 创建模式 -->
      <div
        v-if="isCreating"
        class="flex items-center gap-2 p-2 bg-bg-secondary rounded-lg border border-border-normal"
      >
        <Input
          v-model="newTagName"
          class="w-32 text-sm"
          :placeholder="t('settings.tags.namePlaceholder') || '标签名称'"
          autofocus
          @keydown="handleKeydown($event, 'create')"
        />
        <div class="flex gap-1">
          <button
            v-for="color in colorOptions"
            :key="color"
            type="button"
            class="w-5 h-5 rounded-full border-2 transition-transform hover:scale-110"
            :class="{ 'border-text-primary scale-110': newTagColor === color, 'border-transparent': newTagColor !== color }"
            :style="{ backgroundColor: color }"
            @click="newTagColor = color"
          />
        </div>
        <button
          type="button"
          class="p-1 text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded"
          @click="confirmCreate"
        >
          <Check class="w-4 h-4" />
        </button>
        <button
          type="button"
          class="p-1 text-text-low hover:bg-bg-hover rounded"
          @click="cancelCreate"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- 添加按钮 -->
      <Button
        v-else-if="allowCreate && canCreateMore"
        variant="ghost"
        size="sm"
        class="rounded-full"
        @click="startCreate"
      >
        <Plus class="w-4 h-4 mr-1" />
        {{ t('settings.tags.add') || '添加标签' }}
      </Button>
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="flex items-center gap-2 text-sm text-red-500"
    >
      <AlertCircle class="w-4 h-4" />
      <span>{{ error }}</span>
    </div>

    <!-- 空状态 -->
    <div
      v-if="modelValue.length === 0 && !isCreating"
      class="text-sm text-text-low"
    >
      {{ placeholder || t('settings.tags.empty') || '暂无标签' }}
    </div>

    <!-- 数量提示 -->
    <div
      v-if="modelValue.length > 0"
      class="text-xs text-text-low"
    >
      {{ modelValue.length }} / {{ maxTags }} {{ t('settings.tags.count') || '个标签' }}
    </div>
  </div>
</template>
