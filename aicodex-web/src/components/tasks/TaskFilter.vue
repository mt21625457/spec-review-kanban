<script setup lang="ts">
/**
 * TaskFilter - Task filtering component for status, tags, and search
 */
import { ref, computed, watch } from 'vue'
import { useUiStore } from '@/stores'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Badge from '@/components/ui/Badge.vue'
import type { TaskStatus } from '@/types'

defineProps<{
  availableTags?: string[]
}>()

const emit = defineEmits<{
  search: [query: string]
  filterStatus: [statuses: TaskStatus[]]
  filterTags: [tags: string[]]
  clear: []
}>()

const uiStore = useUiStore()

// Local state
const searchQuery = ref('')
const showStatusDropdown = ref(false)
const showTagDropdown = ref(false)

// Status options - using correct TaskStatus values
const statusOptions: Array<{ value: TaskStatus; label: string; color: string }> = [
  { value: 'todo', label: '待办', color: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200' },
  { value: 'inprogress', label: '进行中', color: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200' },
  { value: 'inreview', label: '评审中', color: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200' },
  { value: 'done', label: '已完成', color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' },
  { value: 'cancelled', label: '已取消', color: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200' },
]

// Computed
const selectedStatuses = computed(() => uiStore.statusFilter as TaskStatus[])
const selectedTags = computed(() => uiStore.tagFilter)
const hasFilters = computed(() =>
  selectedStatuses.value.length > 0 ||
  selectedTags.value.length > 0 ||
  searchQuery.value.trim() !== ''
)

// Methods
const handleSearch = () => {
  uiStore.setSearchQuery(searchQuery.value)
  emit('search', searchQuery.value)
}

const toggleStatus = (status: TaskStatus) => {
  const current = [...selectedStatuses.value]
  const index = current.indexOf(status)
  if (index === -1) {
    current.push(status)
  } else {
    current.splice(index, 1)
  }
  uiStore.setStatusFilter(current)
  emit('filterStatus', current)
}

const toggleTag = (tag: string) => {
  const current = [...selectedTags.value]
  const index = current.indexOf(tag)
  if (index === -1) {
    current.push(tag)
  } else {
    current.splice(index, 1)
  }
  uiStore.setTagFilter(current)
  emit('filterTags', current)
}

const clearFilters = () => {
  searchQuery.value = ''
  uiStore.clearFilters()
  emit('clear')
}

// Sync search query with store
watch(() => uiStore.searchQuery, (newQuery) => {
  searchQuery.value = newQuery
}, { immediate: true })
</script>

<template>
  <div class="flex flex-wrap items-center gap-3">
    <!-- Search Input -->
    <div class="relative flex-1 min-w-[200px] max-w-md">
      <Input
        v-model="searchQuery"
        type="text"
        placeholder="搜索任务..."
        class="pl-9"
        @keyup.enter="handleSearch"
      />
      <svg
        class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>

    <!-- Status Filter -->
    <div class="relative">
      <Button
        variant="secondary"
        size="sm"
        @click="showStatusDropdown = !showStatusDropdown"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
        </svg>
        状态
        <span v-if="selectedStatuses.length" class="ml-1 px-1.5 py-0.5 text-xs bg-brand text-white rounded-full">
          {{ selectedStatuses.length }}
        </span>
      </Button>

      <!-- Status Dropdown -->
      <Transition
        enter-active-class="transition ease-out duration-100"
        enter-from-class="transform opacity-0 scale-95"
        enter-to-class="transform opacity-100 scale-100"
        leave-active-class="transition ease-in duration-75"
        leave-from-class="transform opacity-100 scale-100"
        leave-to-class="transform opacity-0 scale-95"
      >
        <div
          v-if="showStatusDropdown"
          class="absolute left-0 mt-2 w-48 bg-bg-primary border border-border-normal rounded-lg shadow-lg z-10"
        >
          <div class="p-2 space-y-1">
            <label
              v-for="status in statusOptions"
              :key="status.value"
              class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-bg-hover cursor-pointer"
            >
              <input
                type="checkbox"
                :checked="selectedStatuses.includes(status.value)"
                class="rounded border-border-normal text-brand focus:ring-brand"
                @change="toggleStatus(status.value)"
              />
              <span :class="['text-xs px-2 py-0.5 rounded', status.color]">
                {{ status.label }}
              </span>
            </label>
          </div>
        </div>
      </Transition>
    </div>

    <!-- Tag Filter -->
    <div v-if="availableTags?.length" class="relative">
      <Button
        variant="secondary"
        size="sm"
        @click="showTagDropdown = !showTagDropdown"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
        </svg>
        标签
        <span v-if="selectedTags.length" class="ml-1 px-1.5 py-0.5 text-xs bg-brand text-white rounded-full">
          {{ selectedTags.length }}
        </span>
      </Button>

      <!-- Tag Dropdown -->
      <Transition
        enter-active-class="transition ease-out duration-100"
        enter-from-class="transform opacity-0 scale-95"
        enter-to-class="transform opacity-100 scale-100"
        leave-active-class="transition ease-in duration-75"
        leave-from-class="transform opacity-100 scale-100"
        leave-to-class="transform opacity-0 scale-95"
      >
        <div
          v-if="showTagDropdown"
          class="absolute left-0 mt-2 w-48 bg-bg-primary border border-border-normal rounded-lg shadow-lg z-10"
        >
          <div class="p-2 space-y-1 max-h-60 overflow-y-auto">
            <label
              v-for="tag in availableTags"
              :key="tag"
              class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-bg-hover cursor-pointer"
            >
              <input
                type="checkbox"
                :checked="selectedTags.includes(tag)"
                class="rounded border-border-normal text-brand focus:ring-brand"
                @change="toggleTag(tag)"
              />
              <span class="text-sm text-text-primary">{{ tag }}</span>
            </label>
          </div>
        </div>
      </Transition>
    </div>

    <!-- Clear Filters -->
    <Button
      v-if="hasFilters"
      variant="ghost"
      size="sm"
      @click="clearFilters"
    >
      <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
      清除筛选
    </Button>

    <!-- Active Filters Display -->
    <div v-if="hasFilters" class="flex flex-wrap gap-2 w-full mt-2">
      <!-- Search Query -->
      <Badge v-if="searchQuery" variant="default" class="flex items-center gap-1">
        搜索: {{ searchQuery }}
        <button type="button" class="ml-1 hover:text-red-500" @click="searchQuery = ''; handleSearch()">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </Badge>

      <!-- Status Badges -->
      <Badge
        v-for="status in selectedStatuses"
        :key="status"
        variant="default"
        class="flex items-center gap-1"
      >
        {{ statusOptions.find(s => s.value === status)?.label }}
        <button type="button" class="ml-1 hover:text-red-500" @click="toggleStatus(status)">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </Badge>

      <!-- Tag Badges -->
      <Badge
        v-for="tag in selectedTags"
        :key="tag"
        variant="default"
        class="flex items-center gap-1"
      >
        {{ tag }}
        <button type="button" class="ml-1 hover:text-red-500" @click="toggleTag(tag)">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </Badge>
    </div>
  </div>

  <!-- Click outside to close dropdowns -->
  <Teleport to="body">
    <div
      v-if="showStatusDropdown || showTagDropdown"
      class="fixed inset-0 z-0"
      @click="showStatusDropdown = false; showTagDropdown = false"
    />
  </Teleport>
</template>
