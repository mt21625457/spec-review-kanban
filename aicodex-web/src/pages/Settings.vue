<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { configApi, type AllSettings, type GiteaSettings, type ReviewSettings, type QueueSettings, type ConnectionTestResult } from '@/lib/api'
import { PageHeader } from '@/components/layout'
import { Card, Button, Input, Select, Loading } from '@/components/ui'

const { t } = useI18n()

const settings = ref<AllSettings | null>(null)
const loading = ref(true)
const saving = ref(false)
const testResult = ref<ConnectionTestResult | null>(null)
const testLoading = ref(false)

// 表单数据
const giteaForm = ref<GiteaSettings>({})
const reviewForm = ref<ReviewSettings>({
  default_agent: 'codex',
  auto_start: true,
  timeout_seconds: 3600,
})
const queueForm = ref<QueueSettings>({
  max_concurrent: 3,
  retry_count: 3,
  retry_delay_seconds: 60,
})

const agentOptions = [
  { value: 'codex', label: 'Codex' },
  { value: 'claude_code', label: 'Claude Code' },
  { value: 'gemini', label: 'Gemini' },
  { value: 'open_code', label: 'OpenCode' },
  { value: 'copilot', label: 'Copilot' },
]

onMounted(async () => {
  await loadSettings()
})

const loadSettings = async () => {
  loading.value = true
  const response = await configApi.getAll()
  if (response.success && response.data) {
    settings.value = response.data
    giteaForm.value = { ...response.data.gitea }
    reviewForm.value = { ...response.data.review }
    queueForm.value = { ...response.data.queue }
  }
  loading.value = false
}

const saveGiteaSettings = async () => {
  saving.value = true
  const response = await configApi.updateGitea(giteaForm.value)
  if (response.success) {
    alert(t('common.success'))
  } else {
    alert(response.error || t('common.error'))
  }
  saving.value = false
}

const testGiteaConnection = async () => {
  testLoading.value = true
  testResult.value = null
  const response = await configApi.testGiteaConnection()
  if (response.data) {
    testResult.value = response.data
  }
  testLoading.value = false
}

const saveReviewSettings = async () => {
  saving.value = true
  const response = await configApi.updateReview(reviewForm.value)
  if (response.success) {
    alert(t('common.success'))
  } else {
    alert(response.error || t('common.error'))
  }
  saving.value = false
}

const saveQueueSettings = async () => {
  saving.value = true
  const response = await configApi.updateQueue(queueForm.value)
  if (response.success) {
    alert(t('common.success'))
  } else {
    alert(response.error || t('common.error'))
  }
  saving.value = false
}
</script>

<template>
  <div class="max-w-2xl">
    <PageHeader :title="t('settings.title')" />

    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <template v-else>
      <!-- Gitea 配置 -->
      <Card class="mb-6">
        <template #header>
          <h3 class="font-semibold text-text-primary">{{ t('settings.gitea.title') }}</h3>
        </template>

        <form @submit.prevent="saveGiteaSettings" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.gitea.apiUrl') }}
            </label>
            <Input
              v-model="giteaForm.api_url"
              type="text"
              placeholder="https://gitea.example.com"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.gitea.token') }}
            </label>
            <Input
              v-model="giteaForm.token"
              type="password"
              placeholder="输入新 Token 以更新"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.gitea.secret') }}
            </label>
            <Input
              v-model="giteaForm.webhook_secret"
              type="password"
              placeholder="输入新 Secret 以更新"
            />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <Button
              variant="secondary"
              type="button"
              :loading="testLoading"
              @click="testGiteaConnection"
            >
              {{ t('settings.gitea.testConnection') }}
            </Button>
            <Button variant="primary" type="submit" :loading="saving">
              {{ t('common.save') }}
            </Button>
          </div>
          <div
            v-if="testResult"
            class="p-3 rounded-lg text-sm"
            :class="{
              'bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-400': testResult.success,
              'bg-red-50 text-red-700 dark:bg-red-900/20 dark:text-red-400': !testResult.success
            }"
          >
            <template v-if="testResult.success">
              ✓ {{ t('settings.gitea.connectionSuccess') }} - 用户: {{ testResult.user?.login }} ({{ testResult.user?.full_name }})
            </template>
            <template v-else>
              ✗ {{ t('settings.gitea.connectionFailed') }}: {{ testResult.error }}
            </template>
          </div>
        </form>
      </Card>

      <!-- 审核配置 -->
      <Card class="mb-6">
        <template #header>
          <h3 class="font-semibold text-text-primary">{{ t('settings.review.title') }}</h3>
        </template>

        <form @submit.prevent="saveReviewSettings" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.review.defaultAgent') }}
            </label>
            <Select v-model="reviewForm.default_agent" :options="agentOptions" />
          </div>
          <div>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="reviewForm.auto_start"
                type="checkbox"
                class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
              />
              <span class="text-sm text-text-primary">{{ t('settings.review.autoStart') }}</span>
            </label>
          </div>
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.review.timeout') }}
            </label>
            <input
              v-model.number="reviewForm.timeout_seconds"
              type="number"
              min="60"
              class="input"
              placeholder="3600"
            />
          </div>
          <div class="flex justify-end pt-2">
            <Button variant="primary" type="submit" :loading="saving">
              {{ t('common.save') }}
            </Button>
          </div>
        </form>
      </Card>

      <!-- 队列配置 -->
      <Card class="mb-6">
        <template #header>
          <h3 class="font-semibold text-text-primary">{{ t('settings.queue.title') }}</h3>
        </template>

        <form @submit.prevent="saveQueueSettings" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.queue.maxConcurrent') }}
            </label>
            <input
              v-model.number="queueForm.max_concurrent"
              type="number"
              min="1"
              max="10"
              class="input"
              placeholder="3"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.queue.retryCount') }}
            </label>
            <input
              v-model.number="queueForm.retry_count"
              type="number"
              min="0"
              max="10"
              class="input"
              placeholder="3"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.queue.retryDelay') }}
            </label>
            <input
              v-model.number="queueForm.retry_delay_seconds"
              type="number"
              min="10"
              class="input"
              placeholder="60"
            />
          </div>
          <div class="flex justify-end pt-2">
            <Button variant="primary" type="submit" :loading="saving">
              {{ t('common.save') }}
            </Button>
          </div>
        </form>
      </Card>
    </template>
  </div>
</template>
