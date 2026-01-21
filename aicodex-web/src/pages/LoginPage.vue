<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLogin } from '@/composables/useAuth'
import { Button, Input } from '@/components/ui'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

const username = ref('')
const password = ref('')
const errorMessage = ref('')
const showPassword = ref(false)
const isLoaded = ref(false)

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

onMounted(() => {
  // 触发入场动画
  setTimeout(() => {
    isLoaded.value = true
  }, 100)
})
</script>

<template>
  <div class="min-h-screen flex bg-bg-primary overflow-hidden">
    <!-- 左侧品牌展示区 -->
    <div class="hidden lg:flex lg:w-1/2 xl:w-3/5 relative overflow-hidden">
      <!-- 动态渐变背景 -->
      <div class="absolute inset-0 bg-gradient-to-br from-brand via-purple-600 to-indigo-800 dark:from-brand dark:via-purple-700 dark:to-indigo-900">
        <!-- 动态装饰形状 -->
        <div class="absolute inset-0 overflow-hidden">
          <!-- 大圆形装饰 -->
          <div
            class="absolute -top-1/4 -left-1/4 w-[800px] h-[800px] rounded-full bg-white/5 animate-float-slow"
          />
          <div
            class="absolute top-1/2 -right-1/4 w-[600px] h-[600px] rounded-full bg-white/5 animate-float-medium"
          />
          <div
            class="absolute -bottom-1/4 left-1/3 w-[500px] h-[500px] rounded-full bg-white/5 animate-float-fast"
          />
          <!-- 小圆点装饰 -->
          <div class="absolute top-20 left-20 w-3 h-3 rounded-full bg-white/20 animate-pulse" />
          <div class="absolute top-40 right-32 w-2 h-2 rounded-full bg-white/30 animate-pulse delay-100" />
          <div class="absolute bottom-32 left-40 w-4 h-4 rounded-full bg-white/15 animate-pulse delay-200" />
          <div class="absolute bottom-48 right-20 w-2 h-2 rounded-full bg-white/25 animate-pulse delay-300" />
        </div>
      </div>

      <!-- 品牌内容 -->
      <div
        class="relative z-10 flex flex-col justify-center px-12 xl:px-20 text-white transition-all duration-1000"
        :class="isLoaded ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-10'"
      >
        <!-- Logo -->
        <div class="flex items-center gap-4 mb-8">
          <div class="w-14 h-14 rounded-2xl bg-white/20 backdrop-blur-sm flex items-center justify-center shadow-lg">
            <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          </div>
          <span class="text-3xl font-bold tracking-tight">AICodeX</span>
        </div>

        <!-- 标语 -->
        <h1 class="text-4xl xl:text-5xl font-bold leading-tight mb-6">
          {{ t('auth.login.brandTitle', 'AI 驱动的') }}
          <br />
          <span class="text-white/90">{{ t('auth.login.brandSubtitle', '智能编码平台') }}</span>
        </h1>

        <p class="text-lg xl:text-xl text-white/75 max-w-md leading-relaxed mb-10">
          {{ t('auth.login.brandDescription', '借助人工智能的力量，让编码变得更加简单、高效、智能。') }}
        </p>

        <!-- 特性列表 -->
        <div class="space-y-4">
          <div class="flex items-center gap-3 text-white/80">
            <div class="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <span>{{ t('auth.login.feature1', '智能代码补全与生成') }}</span>
          </div>
          <div class="flex items-center gap-3 text-white/80">
            <div class="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <span>{{ t('auth.login.feature2', '多实例协同工作空间') }}</span>
          </div>
          <div class="flex items-center gap-3 text-white/80">
            <div class="w-8 h-8 rounded-full bg-white/20 flex items-center justify-center">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <span>{{ t('auth.login.feature3', '安全可靠的企业级部署') }}</span>
          </div>
        </div>
      </div>

      <!-- 底部装饰线 -->
      <div class="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-white/20 to-transparent" />
    </div>

    <!-- 右侧登录表单区 -->
    <div class="flex-1 flex items-center justify-center p-6 lg:p-12 relative">
      <!-- 背景装饰（移动端可见） -->
      <div class="absolute inset-0 lg:hidden overflow-hidden">
        <div class="absolute -top-1/2 -right-1/2 w-full h-full rounded-full bg-brand/5" />
        <div class="absolute -bottom-1/2 -left-1/2 w-full h-full rounded-full bg-purple-500/5" />
      </div>

      <div
        class="w-full max-w-md relative z-10 transition-all duration-700 delay-200"
        :class="isLoaded ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'"
      >
        <!-- 移动端 Logo -->
        <div class="lg:hidden text-center mb-8">
          <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-brand/10 mb-4">
            <svg class="w-8 h-8 text-brand" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          </div>
          <h2 class="text-2xl font-bold text-text-primary">AICodeX</h2>
        </div>

        <!-- 悬浮登录卡片 -->
        <div class="bg-bg-secondary/80 dark:bg-bg-secondary/60 backdrop-blur-xl rounded-2xl shadow-xl border border-white/20 dark:border-white/5 p-8 lg:p-10">
          <!-- 卡片头部 -->
          <div class="text-center mb-8">
            <h1 class="text-2xl lg:text-3xl font-bold text-text-primary mb-2">
              {{ t('auth.login.title') }}
            </h1>
            <p class="text-text-muted">
              {{ t('auth.login.subtitle') }}
            </p>
          </div>

          <!-- 登录表单 -->
          <form @submit.prevent="handleSubmit" class="space-y-6">
            <!-- 错误提示 -->
            <Transition
              enter-active-class="transition-all duration-300 ease-out"
              enter-from-class="opacity-0 -translate-y-2"
              enter-to-class="opacity-100 translate-y-0"
              leave-active-class="transition-all duration-200 ease-in"
              leave-from-class="opacity-100 translate-y-0"
              leave-to-class="opacity-0 -translate-y-2"
            >
              <div
                v-if="errorMessage"
                class="p-4 bg-error/10 border border-error/20 rounded-xl text-error text-sm flex items-center gap-3"
              >
                <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>{{ errorMessage }}</span>
              </div>
            </Transition>

            <!-- 用户名 -->
            <div class="space-y-2">
              <label for="username" class="block text-sm font-medium text-text-primary">
                {{ t('auth.login.username') }}
              </label>
              <div class="relative group">
                <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-text-muted group-focus-within:text-brand transition-colors">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                  </svg>
                </div>
                <Input
                  id="username"
                  v-model="username"
                  type="text"
                  :placeholder="t('auth.login.usernamePlaceholder')"
                  autocomplete="username"
                  class="pl-12 h-12 bg-bg-primary/50 dark:bg-bg-primary/30 border-border hover:border-brand/50 focus:border-brand focus:ring-2 focus:ring-brand/20 transition-all duration-200"
                />
              </div>
            </div>

            <!-- 密码 -->
            <div class="space-y-2">
              <label for="password" class="block text-sm font-medium text-text-primary">
                {{ t('auth.login.password') }}
              </label>
              <div class="relative group">
                <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-text-muted group-focus-within:text-brand transition-colors">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                  </svg>
                </div>
                <Input
                  id="password"
                  v-model="password"
                  :type="showPassword ? 'text' : 'password'"
                  :placeholder="t('auth.login.passwordPlaceholder')"
                  autocomplete="current-password"
                  class="pl-12 pr-12 h-12 bg-bg-primary/50 dark:bg-bg-primary/30 border-border hover:border-brand/50 focus:border-brand focus:ring-2 focus:ring-brand/20 transition-all duration-200"
                />
                <button
                  type="button"
                  class="absolute right-4 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary focus:outline-none focus:text-brand transition-colors"
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
              class="w-full h-12 text-base font-semibold justify-center rounded-xl shadow-lg shadow-brand/25 hover:shadow-xl hover:shadow-brand/30 hover:-translate-y-0.5 active:translate-y-0 transition-all duration-200"
              :disabled="!isFormValid"
              :loading="loginMutation.isPending.value"
            >
              <template v-if="!loginMutation.isPending.value">
                <span>{{ t('auth.login.submit') }}</span>
                <svg class="w-5 h-5 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
                </svg>
              </template>
            </Button>
          </form>

          <!-- 分隔线 -->
          <div class="relative my-8">
            <div class="absolute inset-0 flex items-center">
              <div class="w-full border-t border-border" />
            </div>
            <div class="relative flex justify-center text-sm">
              <span class="px-4 bg-bg-secondary/80 dark:bg-bg-secondary/60 text-text-muted">
                {{ t('auth.login.or', '或') }}
              </span>
            </div>
          </div>

          <!-- 其他登录方式占位 -->
          <div class="grid grid-cols-3 gap-3">
            <button
              type="button"
              class="flex items-center justify-center h-11 rounded-xl border border-border bg-bg-primary/50 hover:bg-bg-hover hover:border-brand/30 transition-all duration-200"
              :title="t('auth.login.ssoGitHub', 'GitHub 登录')"
            >
              <svg class="w-5 h-5 text-text-normal" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
            </button>
            <button
              type="button"
              class="flex items-center justify-center h-11 rounded-xl border border-border bg-bg-primary/50 hover:bg-bg-hover hover:border-brand/30 transition-all duration-200"
              :title="t('auth.login.ssoGoogle', 'Google 登录')"
            >
              <svg class="w-5 h-5" viewBox="0 0 24 24">
                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
              </svg>
            </button>
            <button
              type="button"
              class="flex items-center justify-center h-11 rounded-xl border border-border bg-bg-primary/50 hover:bg-bg-hover hover:border-brand/30 transition-all duration-200"
              :title="t('auth.login.ssoSSO', 'SSO 登录')"
            >
              <svg class="w-5 h-5 text-text-normal" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
              </svg>
            </button>
          </div>
        </div>

        <!-- 底部信息 -->
        <p class="mt-8 text-center text-sm text-text-muted">
          {{ t('auth.login.footer') }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 浮动动画 */
@keyframes float-slow {
  0%, 100% {
    transform: translate(0, 0) rotate(0deg);
  }
  33% {
    transform: translate(30px, -30px) rotate(5deg);
  }
  66% {
    transform: translate(-20px, 20px) rotate(-5deg);
  }
}

@keyframes float-medium {
  0%, 100% {
    transform: translate(0, 0) rotate(0deg);
  }
  33% {
    transform: translate(-40px, 20px) rotate(-8deg);
  }
  66% {
    transform: translate(30px, -40px) rotate(8deg);
  }
}

@keyframes float-fast {
  0%, 100% {
    transform: translate(0, 0) rotate(0deg);
  }
  33% {
    transform: translate(20px, 30px) rotate(10deg);
  }
  66% {
    transform: translate(-30px, -20px) rotate(-10deg);
  }
}

.animate-float-slow {
  animation: float-slow 20s ease-in-out infinite;
}

.animate-float-medium {
  animation: float-medium 15s ease-in-out infinite;
}

.animate-float-fast {
  animation: float-fast 12s ease-in-out infinite;
}

/* 延迟动画类 */
.delay-100 {
  animation-delay: 0.1s;
}

.delay-200 {
  animation-delay: 0.2s;
}

.delay-300 {
  animation-delay: 0.3s;
}

/* 输入框聚焦时的发光效果 */
:deep(input:focus) {
  box-shadow: 0 0 0 3px hsl(var(--brand) / 0.1);
}

/* 按钮禁用状态 */
:deep(button[disabled]) {
  cursor: not-allowed;
  opacity: 0.6;
}
</style>
