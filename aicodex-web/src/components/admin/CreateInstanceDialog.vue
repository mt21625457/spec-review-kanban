<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, Button, Input } from '@/components/ui'
import { useCreateInstance } from '@/composables/useInstances'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const createMutation = useCreateInstance()

const form = ref({
  name: '',
  description: '',
  auto_start: true,
  max_users: null as number | null,
})

const errorMessage = ref('')

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    form.value = {
      name: '',
      description: '',
      auto_start: true,
      max_users: null,
    }
    errorMessage.value = ''
  }
})

const handleSubmit = async () => {
  errorMessage.value = ''

  if (!form.value.name.trim()) {
    errorMessage.value = '请输入实例名称'
    return
  }

  try {
    await createMutation.mutateAsync({
      name: form.value.name.trim(),
      description: form.value.description.trim() || undefined,
      auto_start: form.value.auto_start,
      max_users: form.value.max_users ?? undefined,
    })
    emit('update:open', false)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '创建实例失败'
  }
}

const handleClose = () => {
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @close="handleClose">
    <template #title>{{ t('admin.instances.createDialog.title') }}</template>

    <form @submit.prevent="handleSubmit" class="space-y-4">
      <!-- 错误提示 -->
      <div
        v-if="errorMessage"
        class="p-3 bg-error/10 border border-error/20 rounded-lg text-error text-sm"
      >
        {{ errorMessage }}
      </div>

      <!-- 实例名称 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.instances.createDialog.name') }} *
        </label>
        <Input
          v-model="form.name"
          :placeholder="t('admin.instances.createDialog.namePlaceholder')"
        />
      </div>

      <!-- 描述 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.instances.createDialog.description') }}
        </label>
        <Input
          v-model="form.description"
          :placeholder="t('admin.instances.createDialog.descriptionPlaceholder')"
        />
      </div>

      <!-- 最大用户数 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.instances.createDialog.maxUsers') }}
        </label>
        <Input
          v-model.number="form.max_users"
          type="number"
          min="1"
          :placeholder="t('admin.instances.createDialog.maxUsersPlaceholder')"
        />
        <p class="mt-1 text-xs text-text-muted">留空表示不限制</p>
      </div>

      <!-- 自动启动 -->
      <div>
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            v-model="form.auto_start"
            type="checkbox"
            class="w-4 h-4 text-brand focus:ring-brand rounded"
          />
          <span class="text-sm text-text-primary">
            {{ t('admin.instances.createDialog.autoStart') }}
          </span>
        </label>
        <p class="mt-1 text-xs text-text-muted ml-6">启用后，系统启动时自动启动此实例</p>
      </div>

      <!-- 按钮 -->
      <div class="flex justify-end gap-3 pt-4">
        <Button variant="ghost" type="button" @click="handleClose">
          {{ t('common.cancel') }}
        </Button>
        <Button
          variant="primary"
          type="submit"
          :loading="createMutation.isPending.value"
        >
          {{ t('common.create') }}
        </Button>
      </div>
    </form>
  </Dialog>
</template>
