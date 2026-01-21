<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLogin } from '@/composables/useAuth'
import { Button, Card, Input } from '@/components/ui'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

const username = ref('')
const password = ref('')
const errorMessage = ref('')
const showPassword = ref(false)

const loginMutation = useLogin()

const isFormValid = computed(() => {
  return username.value.trim().length > 0 && password.value.length >= 6
})

const handleSubmit = async () => {
  if (!isFormValid.value) return

  errorMessage.value = ''

  try {
    await loginMutation.mutateAsync({
      username: username.value.trim(),
      password: password.value,
    })

    // 登录成功，跳转到之前的页面或首页
    const redirect = (route.query.redirect as string) || '/dashboard'
    router.push(redirect)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '登录失败，请检查用户名和密码'
  }
}

const togglePasswordVisibility = () => {
  showPassword.value = !showPassword.value
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-bg-primary px-4">
    <div class="w-full max-w-md">
      <!-- Logo 和标题 -->
      <div class="text-center mb-8">
        <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-brand/10 mb-4">
          <svg class="w-8 h-8 text-brand" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
        </div>
        <h1 class="text-2xl font-bold text-text-primary">{{ t('auth.login.title') }}</h1>
        <p class="mt-2 text-text-muted">{{ t('auth.login.subtitle') }}</p>
      </div>

      <!-- 登录表单 -->
      <Card>
        <form @submit.prevent="handleSubmit" class="space-y-6">
          <!-- 错误提示 -->
          <div
            v-if="errorMessage"
            class="p-3 bg-error/10 border border-error/20 rounded-lg text-error text-sm"
          >
            {{ errorMessage }}
          </div>

          <!-- 用户名 -->
          <div>
            <label for="username" class="block text-sm font-medium text-text-primary mb-1.5">
              {{ t('auth.login.username') }}
            </label>
            <Input
              id="username"
              v-model="username"
              type="text"
              :placeholder="t('auth.login.usernamePlaceholder')"
              autocomplete="username"
            />
          </div>

          <!-- 密码 -->
          <div>
            <label for="password" class="block text-sm font-medium text-text-primary mb-1.5">
              {{ t('auth.login.password') }}
            </label>
            <div class="relative">
              <Input
                id="password"
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                :placeholder="t('auth.login.passwordPlaceholder')"
                autocomplete="current-password"
                class="pr-10"
              />
              <button
                type="button"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary"
                @click="togglePasswordVisibility"
              >
                <svg v-if="showPassword" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
                <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              </button>
            </div>
          </div>

          <!-- 登录按钮 -->
          <Button
            type="submit"
            variant="primary"
            class="w-full justify-center"
            :disabled="!isFormValid"
            :loading="loginMutation.isPending.value"
          >
            {{ t('auth.login.submit') }}
          </Button>
        </form>
      </Card>

      <!-- 底部信息 -->
      <p class="mt-6 text-center text-sm text-text-muted">
        {{ t('auth.login.footer') }}
      </p>
    </div>
  </div>
</template>
