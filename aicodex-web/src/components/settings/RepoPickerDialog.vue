<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, Button, Input, Loading } from '@/components/ui'
import { fileSystemApi, type DirectoryEntry } from '@/lib/api'
import { FolderOpen, ChevronRight, ChevronLeft, Check } from 'lucide-vue-next'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'select': [repo: { path: string; displayName: string }]
}>()

const { t } = useI18n()

// 当前路径和目录列表
const currentPath = ref<string>('')
const entries = ref<DirectoryEntry[]>([])
const parentPath = ref<string | null>(null)
const loading = ref(false)

// 选中的仓库
const selectedRepo = ref<DirectoryEntry | null>(null)

// 自定义显示名称
const displayName = ref('')

// 搜索过滤
const searchQuery = ref('')

// 过滤后的目录列表
const filteredEntries = computed(() => {
  if (!searchQuery.value) return entries.value
  const query = searchQuery.value.toLowerCase()
  return entries.value.filter(e => e.name.toLowerCase().includes(query))
})

// 加载目录
const loadDirectory = async (path?: string) => {
  loading.value = true
  try {
    const response = await fileSystemApi.listDirectory(path)
    currentPath.value = response.path
    entries.value = response.entries.filter(e => e.is_dir)
    parentPath.value = response.parent_path
    selectedRepo.value = null
  } catch (err) {
    console.error('Failed to load directory:', err)
  } finally {
    loading.value = false
  }
}

// 进入目录
const enterDirectory = (entry: DirectoryEntry) => {
  if (entry.is_git_repo) {
    // 选中仓库
    selectedRepo.value = entry
    displayName.value = entry.name
  } else {
    loadDirectory(entry.path)
  }
}

// 返回上级目录
const goBack = () => {
  if (parentPath.value !== null) {
    loadDirectory(parentPath.value || undefined)
  }
}

// 确认选择
const confirmSelection = () => {
  if (selectedRepo.value) {
    emit('select', {
      path: selectedRepo.value.path,
      displayName: displayName.value || selectedRepo.value.name,
    })
    emit('update:open', false)
  }
}

// 取消
const cancel = () => {
  emit('update:open', false)
}

// 初始加载
onMounted(() => {
  if (props.open) {
    loadDirectory()
  }
})

// 监听对话框打开
const handleOpenChange = (open: boolean) => {
  emit('update:open', open)
  if (open && entries.value.length === 0) {
    loadDirectory()
  }
}
</script>

<template>
  <Dialog :open="open" @update:open="handleOpenChange">
    <div class="w-[600px] max-h-[80vh] flex flex-col">
      <!-- 标题 -->
      <div class="px-6 py-4 border-b border-border-normal">
        <h2 class="text-lg font-semibold text-text-primary">
          {{ t('settings.projects.selectRepo') || '选择仓库' }}
        </h2>
      </div>

      <!-- 路径导航 -->
      <div class="px-6 py-3 bg-bg-secondary border-b border-border-normal">
        <div class="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            :disabled="parentPath === null"
            @click="goBack"
          >
            <ChevronLeft class="w-4 h-4" />
          </Button>
          <div class="flex-1 font-mono text-sm text-text-normal truncate">
            {{ currentPath || '/' }}
          </div>
        </div>
      </div>

      <!-- 搜索框 -->
      <div class="px-6 py-3 border-b border-border-normal">
        <Input
          v-model="searchQuery"
          :placeholder="t('common.search') || '搜索...'"
          class="w-full"
        />
      </div>

      <!-- 目录列表 -->
      <div class="flex-1 overflow-auto min-h-[300px]">
        <div v-if="loading" class="py-12">
          <Loading />
        </div>
        <div v-else-if="filteredEntries.length === 0" class="py-12 text-center text-text-low">
          {{ t('common.noData') || '无数据' }}
        </div>
        <div v-else class="divide-y divide-border-normal">
          <button
            v-for="entry in filteredEntries"
            :key="entry.path"
            class="w-full flex items-center gap-3 px-6 py-3 hover:bg-bg-hover transition-colors text-left"
            :class="{ 'bg-brand/10': selectedRepo?.path === entry.path }"
            @click="enterDirectory(entry)"
          >
            <FolderOpen
              class="w-5 h-5 flex-shrink-0"
              :class="entry.is_git_repo ? 'text-brand' : 'text-text-low'"
            />
            <div class="flex-1 min-w-0">
              <div class="font-medium text-text-primary truncate">
                {{ entry.name }}
              </div>
              <div v-if="entry.is_git_repo" class="text-xs text-brand">
                Git 仓库
              </div>
            </div>
            <ChevronRight v-if="!entry.is_git_repo" class="w-4 h-4 text-text-low" />
            <Check v-else-if="selectedRepo?.path === entry.path" class="w-4 h-4 text-brand" />
          </button>
        </div>
      </div>

      <!-- 选中的仓库信息 -->
      <div v-if="selectedRepo" class="px-6 py-4 bg-bg-secondary border-t border-border-normal">
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.repos.path') || '仓库路径' }}
            </label>
            <div class="font-mono text-sm text-text-normal bg-bg-primary px-3 py-2 rounded-lg">
              {{ selectedRepo.path }}
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.repos.displayName') || '显示名称' }}
            </label>
            <Input
              v-model="displayName"
              :placeholder="selectedRepo.name"
            />
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="px-6 py-4 border-t border-border-normal flex justify-end gap-3">
        <Button variant="secondary" @click="cancel">
          {{ t('common.cancel') }}
        </Button>
        <Button variant="primary" :disabled="!selectedRepo" @click="confirmSelection">
          {{ t('common.confirm') || '确定' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
