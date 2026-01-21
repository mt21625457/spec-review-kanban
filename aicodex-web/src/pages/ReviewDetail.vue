<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { reviewApi, type ReviewRun, type ReviewEvent } from '@/lib/api'
import { Card, Badge, Button, Loading, Tabs, TabPanel } from '@/components/ui'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const review = ref<ReviewRun | null>(null)
const events = ref<ReviewEvent[]>([])
const loading = ref(true)
const actionLoading = ref(false)
const activeTab = ref('events')

const reviewId = route.params.id as string

const tabs = [
  { name: 'events', label: t('reviews.detail.tabs.events') },
  { name: 'logs', label: t('reviews.detail.tabs.logs') },
  { name: 'result', label: t('reviews.detail.tabs.result') },
]

onMounted(async () => {
  await loadReview()
})

const loadReview = async () => {
  loading.value = true
  const response = await reviewApi.get(reviewId)
  if (response.success && response.data) {
    review.value = response.data.review
    events.value = response.data.events
  }
  loading.value = false
}

const handleRerun = async () => {
  if (!confirm('确定要重新运行此审核吗？')) return
  actionLoading.value = true
  const response = await reviewApi.rerun(reviewId)
  if (response.success) {
    alert('已创建新的审核运行')
    if (response.data) {
      router.push(`/reviews/${response.data.id}`)
    }
  } else {
    alert(response.error || '操作失败')
  }
  actionLoading.value = false
}

const handleCancel = async () => {
  if (!confirm('确定要取消此审核吗？')) return
  actionLoading.value = true
  const response = await reviewApi.cancel(reviewId)
  if (response.success) {
    await loadReview()
  } else {
    alert(response.error || '操作失败')
  }
  actionLoading.value = false
}

const getStatusVariant = (status: string) => {
  switch (status) {
    case 'pending': return 'pending'
    case 'running': return 'running'
    case 'completed': return 'completed'
    case 'failed': return 'failed'
    case 'cancelled': return 'cancelled'
    default: return 'default'
  }
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN')
}

const formatEventData = (data: string) => {
  try {
    return JSON.stringify(JSON.parse(data), null, 2)
  } catch {
    return data
  }
}
</script>

<template>
  <div>
    <!-- 返回链接 -->
    <router-link
      to="/reviews"
      class="inline-flex items-center gap-1 text-text-muted hover:text-brand mb-4"
    >
      <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      {{ t('reviews.detail.back') }}
    </router-link>

    <!-- 加载状态 -->
    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <template v-else-if="review">
      <!-- 标题栏 -->
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
        <div class="flex items-center gap-3">
          <h1 class="text-2xl font-bold text-text-primary">
            PR #{{ review.gitea_pr_number }}
          </h1>
          <Badge :variant="getStatusVariant(review.status)">
            {{ t(`reviews.status.${review.status}`) }}
          </Badge>
        </div>
        <div class="flex gap-2">
          <Button
            v-if="['pending', 'running'].includes(review.status)"
            variant="secondary"
            :loading="actionLoading"
            @click="handleCancel"
          >
            {{ t('reviews.detail.cancel') }}
          </Button>
          <Button
            variant="primary"
            :loading="actionLoading"
            @click="handleRerun"
          >
            {{ t('reviews.detail.rerun') }}
          </Button>
        </div>
      </div>

      <!-- 信息卡片 -->
      <div class="grid md:grid-cols-2 gap-4 mb-6">
        <Card>
          <template #header>
            <h3 class="font-semibold text-text-primary">{{ t('reviews.detail.basicInfo') }}</h3>
          </template>
          <div class="space-y-4">
            <div>
              <div class="text-xs text-text-muted uppercase mb-1">{{ t('reviews.detail.repo') }}</div>
              <div class="text-text-primary">{{ review.gitea_repo || '-' }}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase mb-1">{{ t('reviews.detail.pr') }}</div>
              <a
                v-if="review.gitea_pr_url"
                :href="review.gitea_pr_url"
                target="_blank"
                class="text-brand hover:underline"
              >
                {{ review.gitea_pr_url }}
              </a>
              <span v-else class="text-text-primary">-</span>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase mb-1">{{ t('reviews.detail.created') }}</div>
              <div class="text-text-primary">{{ formatDate(review.created_at) }}</div>
            </div>
          </div>
        </Card>

        <Card>
          <template #header>
            <h3 class="font-semibold text-text-primary">{{ t('reviews.detail.executionInfo') }}</h3>
          </template>
          <div class="space-y-4">
            <div>
              <div class="text-xs text-text-muted uppercase mb-1">{{ t('reviews.detail.agent') }}</div>
              <div class="text-text-primary">{{ review.agent_type || '-' }}</div>
            </div>
            <div>
              <div class="text-xs text-text-muted uppercase mb-1">{{ t('reviews.detail.started') }}</div>
              <div class="text-text-primary">{{ review.started_at ? formatDate(review.started_at) : '-' }}</div>
            </div>
            <div v-if="review.vibe_task_id">
              <div class="text-xs text-text-muted uppercase mb-1">Vibe Task ID</div>
              <div class="text-text-primary font-mono text-sm">{{ review.vibe_task_id }}</div>
            </div>
          </div>
        </Card>
      </div>

      <!-- 错误信息 -->
      <Card v-if="review.error_message" class="mb-6 border-red-200 bg-red-50 dark:bg-red-900/10">
        <div class="text-red-600 dark:text-red-400">
          <div class="font-semibold mb-2">错误信息</div>
          <div class="text-sm">{{ review.error_message }}</div>
        </div>
      </Card>

      <!-- Tabs -->
      <Card>
        <Tabs v-model="activeTab" :tabs="tabs">
          <!-- 事件历史 -->
          <TabPanel name="events">
            <div v-if="events.length === 0" class="py-8 text-center text-text-muted">
              暂无事件
            </div>
            <div v-else class="space-y-3">
              <div
                v-for="event in events"
                :key="event.id"
                class="p-4 bg-bg-secondary rounded-lg"
              >
                <div class="flex items-center justify-between mb-2">
                  <span class="font-medium text-text-primary">{{ event.event_type }}</span>
                  <span class="text-xs text-text-muted">{{ formatDate(event.created_at) }}</span>
                </div>
                <div v-if="event.event_data" class="mt-2">
                  <pre class="text-xs bg-bg-tertiary p-3 rounded overflow-x-auto text-text-muted">{{ formatEventData(event.event_data) }}</pre>
                </div>
              </div>
            </div>
          </TabPanel>

          <!-- 执行日志 -->
          <TabPanel name="logs">
            <div class="py-8 text-center text-text-muted">
              暂无日志
            </div>
          </TabPanel>

          <!-- 审核结果 -->
          <TabPanel name="result">
            <div class="py-8 text-center text-text-muted">
              暂无结果
            </div>
          </TabPanel>
        </Tabs>
      </Card>
    </template>
  </div>
</template>
