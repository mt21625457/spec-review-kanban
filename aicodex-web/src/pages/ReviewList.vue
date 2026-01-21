<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { reviewApi, type ReviewRun } from '@/lib/api'
import { PageHeader } from '@/components/layout'
import { Card, Badge, Button, Loading, Input, Select } from '@/components/ui'

const { t } = useI18n()

const reviews = ref<ReviewRun[]>([])
const loading = ref(true)
const searchQuery = ref('')
const statusFilter = ref<string>('all')

const statusOptions = [
  { value: 'all', label: t('reviews.status.all') },
  { value: 'pending', label: t('reviews.status.pending') },
  { value: 'running', label: t('reviews.status.running') },
  { value: 'completed', label: t('reviews.status.completed') },
  { value: 'failed', label: t('reviews.status.failed') },
  { value: 'cancelled', label: t('reviews.status.cancelled') },
]

onMounted(async () => {
  await loadReviews()
})

const loadReviews = async () => {
  loading.value = true
  const response = await reviewApi.list(100)
  if (response.success && response.data) {
    reviews.value = response.data
  }
  loading.value = false
}

const filteredReviews = computed(() => {
  let result = reviews.value

  // 状态筛选
  if (statusFilter.value !== 'all') {
    result = result.filter(r => r.status === statusFilter.value)
  }

  // 搜索筛选
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(r =>
      r.gitea_pr_number?.toString().includes(query) ||
      r.gitea_repo?.toLowerCase().includes(query)
    )
  }

  return result
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
    <PageHeader :title="t('reviews.title')">
      <template #actions>
        <Button variant="secondary" @click="loadReviews">
          <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ t('reviews.refresh') }}
        </Button>
      </template>
    </PageHeader>

    <!-- 筛选栏 -->
    <div class="flex flex-col sm:flex-row gap-4 mb-6">
      <div class="flex-1">
        <Input
          v-model="searchQuery"
          :placeholder="t('reviews.search')"
        />
      </div>
      <div class="w-full sm:w-48">
        <Select
          v-model="statusFilter"
          :options="statusOptions"
        />
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <!-- 空状态 -->
    <Card v-else-if="filteredReviews.length === 0">
      <div class="py-12 text-center text-text-muted">
        {{ t('reviews.empty') }}
      </div>
    </Card>

    <!-- 审核列表 -->
    <Card v-else>
      <div class="overflow-x-auto -mx-4 -my-4">
        <table class="w-full">
          <thead>
            <tr class="border-b border-border-normal bg-bg-secondary">
              <th class="px-4 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
                PR
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
                {{ t('reviews.status.all').replace('全部', '') }}状态
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
                {{ t('reviews.detail.created') }}
              </th>
              <th class="px-4 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider hidden sm:table-cell">
                完成时间
              </th>
              <th class="px-4 py-3 text-right text-xs font-medium text-text-muted uppercase tracking-wider">
                操作
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border-normal">
            <tr
              v-for="review in filteredReviews"
              :key="review.id"
              class="hover:bg-bg-hover transition-colors"
            >
              <td class="px-4 py-3">
                <a
                  v-if="review.gitea_pr_url"
                  :href="review.gitea_pr_url"
                  target="_blank"
                  class="font-medium text-brand hover:underline"
                >
                  PR #{{ review.gitea_pr_number }}
                </a>
                <span v-else class="font-medium text-text-primary">
                  PR #{{ review.gitea_pr_number }}
                </span>
              </td>
              <td class="px-4 py-3">
                <Badge :variant="getStatusVariant(review.status)">
                  {{ t(`reviews.status.${review.status}`) }}
                </Badge>
              </td>
              <td class="px-4 py-3 text-sm text-text-muted">
                {{ formatDate(review.created_at) }}
              </td>
              <td class="px-4 py-3 text-sm text-text-muted hidden sm:table-cell">
                {{ review.completed_at ? formatDate(review.completed_at) : '-' }}
              </td>
              <td class="px-4 py-3 text-right">
                <router-link
                  :to="`/reviews/${review.id}`"
                  class="text-sm text-brand hover:underline"
                >
                  查看详情
                </router-link>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </Card>
  </div>
</template>
