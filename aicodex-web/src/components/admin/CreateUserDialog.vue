<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, Button, Input } from '@/components/ui'
import { useCreateUser } from '@/composables/useUsers'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const createUserMutation = useCreateUser()

const form = ref({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
  display_name: '',
  role: 'user' as 'admin' | 'user',
})

const errorMessage = ref('')

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    // 重置表单
    form.value = {
      username: '',
      email: '',
      password: '',
      confirmPassword: '',
      display_name: '',
      role: 'user',
    }
    errorMessage.value = ''
  }
})

const handleSubmit = async () => {
  errorMessage.value = ''

  // 验证
  if (!form.value.username.trim()) {
    errorMessage.value = '请输入用户名'
    return
  }
  if (form.value.password.length < 6) {
    errorMessage.value = '密码长度至少 6 位'
    return
  }
  if (form.value.password !== form.value.confirmPassword) {
    errorMessage.value = '两次输入的密码不一致'
    return
  }

  try {
    await createUserMutation.mutateAsync({
      username: form.value.username.trim(),
      email: form.value.email.trim() || undefined,
      password: form.value.password,
      display_name: form.value.display_name.trim() || undefined,
      role: form.value.role,
    })
    emit('update:open', false)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '创建用户失败'
  }
}

const handleClose = () => {
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @close="handleClose">
    <template #title>{{ t('admin.users.createDialog.title') }}</template>

    <form @submit.prevent="handleSubmit" class="space-y-4">
      <!-- 错误提示 -->
      <div
        v-if="errorMessage"
        class="p-3 bg-error/10 border border-error/20 rounded-lg text-error text-sm"
      >
        {{ errorMessage }}
      </div>

      <!-- 用户名 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.users.createDialog.username') }} *
        </label>
        <Input
          v-model="form.username"
          :placeholder="t('admin.users.createDialog.usernamePlaceholder')"
        />
      </div>

      <!-- 显示名称 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.users.createDialog.displayName') }}
        </label>
        <Input
          v-model="form.display_name"
          :placeholder="t('admin.users.createDialog.displayNamePlaceholder')"
        />
      </div>

      <!-- 邮箱 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.users.createDialog.email') }}
        </label>
        <Input
          v-model="form.email"
          type="email"
          :placeholder="t('admin.users.createDialog.emailPlaceholder')"
        />
      </div>

      <!-- 密码 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.users.createDialog.password') }} *
        </label>
        <Input
          v-model="form.password"
          type="password"
          :placeholder="t('admin.users.createDialog.passwordPlaceholder')"
        />
      </div>

      <!-- 确认密码 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.users.createDialog.confirmPassword') }} *
        </label>
        <Input
          v-model="form.confirmPassword"
          type="password"
          :placeholder="t('admin.users.createDialog.confirmPasswordPlaceholder')"
        />
      </div>

      <!-- 角色 -->
      <div>
        <label class="block text-sm font-medium text-text-primary mb-1.5">
          {{ t('admin.users.createDialog.role') }}
        </label>
        <div class="flex gap-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="form.role"
              type="radio"
              value="user"
              class="text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">普通用户</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="form.role"
              type="radio"
              value="admin"
              class="text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">管理员</span>
          </label>
        </div>
      </div>

      <!-- 按钮 -->
      <div class="flex justify-end gap-3 pt-4">
        <Button variant="ghost" type="button" @click="handleClose">
          {{ t('common.cancel') }}
        </Button>
        <Button
          variant="primary"
          type="submit"
          :loading="createUserMutation.isPending.value"
        >
          {{ t('common.create') }}
        </Button>
      </div>
    </form>
  </Dialog>
</template>
