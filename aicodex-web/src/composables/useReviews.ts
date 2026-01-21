import { ref } from 'vue'

const API_BASE = '/api'

// 类型定义
export interface ReviewRun {
  id: string
  repo_mapping_id: string
  gitea_pr_number: number
  gitea_pr_title?: string
  gitea_repo?: string
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
  error_message?: string
  vibe_task_id?: string
  vibe_workspace_id?: string
  created_at: string
  updated_at: string
  started_at?: string
  completed_at?: string
}

export interface ReviewEvent {
  id: string
  review_run_id: string
  event_type: string
  message: string
  created_at: string
}

async function request<T>(url: string, options?: RequestInit): Promise<{ data: T | null; error: string | null }> {
  try {
    const response = await fetch(API_BASE + url, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    })

    const json = await response.json()

    if (json.success) {
      return { data: json.data, error: null }
    } else {
      return { data: null, error: json.error || '请求失败' }
    }
  } catch (e) {
    return { data: null, error: e instanceof Error ? e.message : '网络错误' }
  }
}

export function useReviews() {
  const reviews = ref<ReviewRun[]>([])
  const currentReview = ref<ReviewRun | null>(null)
  const events = ref<ReviewEvent[]>([])
  const isLoading = ref(false)

  const fetchReviews = async (params?: { limit?: number; status?: string }) => {
    isLoading.value = true
    const queryString = params ? `?${new URLSearchParams(params as Record<string, string>)}` : ''
    const { data, error } = await request<ReviewRun[]>(`/reviews${queryString}`)
    if (data) {
      reviews.value = data
    }
    isLoading.value = false
    return { data, error }
  }

  const fetchReview = async (id: string) => {
    isLoading.value = true
    const { data, error } = await request<{ review: ReviewRun; events: ReviewEvent[] }>(`/reviews/${id}`)
    if (data) {
      currentReview.value = data.review
      events.value = data.events
    }
    isLoading.value = false
    return { data, error }
  }

  const rerunReview = async (id: string) => {
    const { data, error } = await request<ReviewRun>(`/reviews/${id}/rerun`, { method: 'POST' })
    if (data) {
      const index = reviews.value.findIndex((r) => r.id === id)
      if (index !== -1) {
        reviews.value[index] = data
      }
      if (currentReview.value?.id === id) {
        currentReview.value = data
      }
    }
    return { data, error }
  }

  const cancelReview = async (id: string) => {
    const { data, error } = await request<ReviewRun>(`/reviews/${id}/cancel`, { method: 'POST' })
    if (data) {
      const index = reviews.value.findIndex((r) => r.id === id)
      if (index !== -1) {
        reviews.value[index] = data
      }
      if (currentReview.value?.id === id) {
        currentReview.value = data
      }
    }
    return { data, error }
  }

  const getStats = () => {
    const total = reviews.value.length
    const running = reviews.value.filter((r) => r.status === 'running').length
    const pending = reviews.value.filter((r) => r.status === 'pending').length
    const completed = reviews.value.filter((r) => r.status === 'completed').length
    const failed = reviews.value.filter((r) => r.status === 'failed').length
    return { total, running, pending, completed, failed }
  }

  return {
    reviews,
    currentReview,
    events,
    isLoading,
    fetchReviews,
    fetchReview,
    rerunReview,
    cancelReview,
    getStats,
  }
}
