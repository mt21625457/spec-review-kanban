<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { reviewApi, type ReviewRun } from '@/lib/api'
import { PageHeader } from '@/components/layout'
import { Card, Badge, Loading } from '@/components/ui'

const { t } = useI18n()

const recentReviews = ref<ReviewRun[]>([])
const loading = ref(true)
const stats = ref({
  total: 0,
  pending: 0,
  running: 0,
  completed: 0,
  failed: 0,
})

onMounted(async () => {
  const response = await reviewApi.list(10)
  if (response.success && response.data) {
    recentReviews.value = response.data
    stats.value.total = response.data.length
    stats.value.pending = response.data.filter(r => r.status === 'pending').length
    stats.value.running = response.data.filter(r => r.status === 'running').length
    stats.value.completed = response.data.filter(r => r.status === 'completed').length
    stats.value.failed = response.data.filter(r => r.status === 'failed').length
  }
  loading.value = false
})

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN')
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
</script>

<template>
  <div>
    <PageHeader :title="t('nav.dashboard')" />

    <!-- 欢迎语 -->
    <div class="mb-8">
      <h2 class="text-2xl font-bold text-text-primary">{{ t('dashboard.welcome') }}</h2>
      <p class="mt-1 text-text-muted">{{ t('dashboard.subtitle') }}</p>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 mb-8">
      <Card>
        <div class="text-center">
          <div class="text-3xl font-bold text-text-primary">{{ stats.total }}</div>
          <div class="text-sm text-text-muted mt-1">{{ t('dashboard.stats.total') }}</div>
        </div>
      </Card>
      <Card>
        <div class="text-center">
          <div class="text-3xl font-bold text-yellow-500">{{ stats.pending }}</div>
          <div class="text-sm text-text-muted mt-1">{{ t('dashboard.stats.pending') }}</div>
        </div>
      </Card>
      <Card>
        <div class="text-center">
          <div class="text-3xl font-bold text-blue-500">{{ stats.running }}</div>
          <div class="text-sm text-text-muted mt-1">{{ t('dashboard.stats.running') }}</div>
        </div>
      </Card>
      <Card>
        <div class="text-center">
          <div class="text-3xl font-bold text-green-500">{{ stats.completed }}</div>
          <div class="text-sm text-text-muted mt-1">{{ t('dashboard.stats.completed') }}</div>
        </div>
      </Card>
      <Card class="col-span-2 md:col-span-1">
        <div class="text-center">
          <div class="text-3xl font-bold text-red-500">{{ stats.failed }}</div>
          <div class="text-sm text-text-muted mt-1">{{ t('reviews.status.failed') }}</div>
        </div>
      </Card>
    </div>

    <!-- 快速入口 -->
    <div class="grid md:grid-cols-2 gap-4 mb-8">
      <router-link to="/reviews" class="block">
        <Card class="h-full hover:shadow-md transition-shadow cursor-pointer">
          <div class="flex items-center gap-4">
            <div class="p-3 bg-brand/10 rounded-lg">
              <svg class="h-6 w-6 text-brand" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
              </svg>
            </div>
            <div class="flex-1">
              <h3 class="font-semibold text-text-primary">{{ t('dashboard.quickActions.reviews') }}</h3>
              <p class="text-sm text-text-muted">{{ t('dashboard.quickActions.reviewsDesc') }}</p>
            </div>
            <svg class="h-5 w-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </Card>
      </router-link>

      <router-link to="/tasks" class="block">
        <Card class="h-full hover:shadow-md transition-shadow cursor-pointer">
          <div class="flex items-center gap-4">
            <div class="p-3 bg-brand/10 rounded-lg">
              <svg class="h-6 w-6 text-brand" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2" />
              </svg>
            </div>
            <div class="flex-1">
              <h3 class="font-semibold text-text-primary">{{ t('dashboard.quickActions.tasks') }}</h3>
              <p class="text-sm text-text-muted">{{ t('dashboard.quickActions.tasksDesc') }}</p>
            </div>
            <svg class="h-5 w-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </Card>
      </router-link>
    </div>

    <!-- 最近审核 -->
    <Card>
      <template #header>
        <div class="flex items-center justify-between">
          <h3 class="font-semibold text-text-primary">{{ t('dashboard.recentReviews') }}</h3>
          <router-link to="/reviews" class="text-sm text-brand hover:underline">
            {{ t('common.back') }} →
          </router-link>
        </div>
      </template>

      <div v-if="loading" class="py-8">
        <Loading />
      </div>
      <div v-else-if="recentReviews.length === 0" class="py-8 text-center text-text-muted">
        {{ t('reviews.empty') }}
      </div>
      <div v-else class="divide-y divide-border-normal -mx-4 -mb-4">
        <router-link
          v-for="review in recentReviews"
          :key="review.id"
          :to="`/reviews/${review.id}`"
          class="flex items-center justify-between px-4 py-3 hover:bg-bg-hover transition-colors"
        >
          <div class="flex items-center gap-3">
            <span class="font-medium text-text-primary">PR #{{ review.gitea_pr_number }}</span>
            <Badge :variant="getStatusVariant(review.status)">
              {{ t(`reviews.status.${review.status}`) }}
            </Badge>
          </div>
          <span class="text-sm text-text-muted">{{ formatDate(review.created_at) }}</span>
        </router-link>
      </div>
    </Card>
  </div>
</template>
