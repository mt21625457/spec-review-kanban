<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Card, Input, Select, Loading } from '@/components/ui'
import { SettingsSaveFooter } from '@/components/settings'
import { useLocalDraft } from '@/composables/useSettingsDraft'
import { reposApi } from '@/lib/api'
import type { Repo, UpdateRepo } from '@/types'
import { Info } from 'lucide-vue-next'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

// 检测操作系统
const isMac = navigator.platform.toLowerCase().includes('mac')
const isWindows = navigator.platform.toLowerCase().includes('win')
const osName = isMac ? 'macOS' : isWindows ? 'Windows' : 'Linux'

// 仓库列表和选中仓库
const repos = ref<Repo[]>([])
const selectedRepoId = ref<string | null>(null)

// 草稿管理
const { draft, hasChanges, saving, success, error, init, update, reset, save } = useLocalDraft<UpdateRepo & { path?: string }>()

// 加载状态
const loading = ref(true)

// 计算当前选中的仓库
const selectedRepo = computed(() =>
  repos.value.find(r => r.id === selectedRepoId.value) || null
)

// 仓库选项
const repoOptions = computed(() =>
  repos.value.map(r => ({
    value: r.id,
    label: r.display_name || r.name
  }))
)

// 加载仓库列表
onMounted(async () => {
  await loadRepos()
})

// 监听 URL 参数变化
watch(() => route.query.repoId, (newId) => {
  if (newId && typeof newId === 'string') {
    selectedRepoId.value = newId
  }
}, { immediate: true })

// 监听选中仓库变化
watch(selectedRepoId, async (newId) => {
  if (newId) {
    router.replace({ query: { ...route.query, repoId: newId } })
    const repo = repos.value.find(r => r.id === newId)
    if (repo) {
      init({
        display_name: repo.display_name,
        path: repo.path,
        dev_server_script: repo.dev_server_script,
        setup_script: repo.setup_script,
        parallel_setup_script: repo.parallel_setup_script,
        cleanup_script: repo.cleanup_script,
        copy_files: repo.copy_files,
      })
    }
  }
})

const loadRepos = async () => {
  loading.value = true
  try {
    const data = await reposApi.list()
    repos.value = data
    // 如果 URL 没有指定仓库，选择第一个
    if (!selectedRepoId.value && data.length > 0) {
      selectedRepoId.value = data[0].id
    }
  } catch (err) {
    console.error('Failed to load repos:', err)
  } finally {
    loading.value = false
  }
}

// 保存仓库配置
const handleSave = async () => {
  if (!selectedRepoId.value || !draft.value) return

  await save(async (data) => {
    const { path, ...updateData } = data
    await reposApi.update(selectedRepoId.value!, updateData)
    // 更新本地列表
    const index = repos.value.findIndex(r => r.id === selectedRepoId.value)
    if (index !== -1) {
      repos.value[index] = {
        ...repos.value[index],
        display_name: updateData.display_name || repos.value[index].display_name,
        dev_server_script: updateData.dev_server_script ?? repos.value[index].dev_server_script,
        setup_script: updateData.setup_script ?? repos.value[index].setup_script,
        parallel_setup_script: updateData.parallel_setup_script ?? repos.value[index].parallel_setup_script,
        cleanup_script: updateData.cleanup_script ?? repos.value[index].cleanup_script,
        copy_files: updateData.copy_files ?? repos.value[index].copy_files,
      }
    }
  })
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-text-primary">
        {{ t('settings.repos.title') }}
      </h2>
    </div>

    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <template v-else>
      <!-- 仓库选择器 -->
      <Card>
        <div class="flex items-center gap-4">
          <label class="text-sm font-medium text-text-primary whitespace-nowrap">
            {{ t('settings.repos.select') }}
          </label>
          <Select
            v-model="selectedRepoId"
            :options="repoOptions"
            :placeholder="t('settings.repos.selectPlaceholder')"
            class="flex-1"
          />
        </div>
      </Card>

      <template v-if="selectedRepo && draft">
        <!-- 常规信息 -->
        <Card>
          <template #header>
            <h3 class="font-semibold text-text-primary">
              {{ t('settings.repos.general') }}
            </h3>
          </template>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.repos.displayName') }}
              </label>
              <Input
                :model-value="draft.display_name || ''"
                @update:model-value="update({ display_name: $event as string })"
                :placeholder="t('settings.repos.displayNamePlaceholder')"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.repos.path') }}
              </label>
              <div class="px-3 py-2 bg-bg-secondary rounded-lg font-mono text-sm text-text-normal">
                {{ draft.path }}
              </div>
            </div>
          </div>
        </Card>

        <!-- 脚本配置 -->
        <Card>
          <template #header>
            <h3 class="font-semibold text-text-primary">
              {{ t('settings.repos.scripts') }}
            </h3>
          </template>

          <div class="space-y-6">
            <!-- Dev Server 脚本 -->
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.repos.devServer') }}
              </label>
              <textarea
                :value="draft.dev_server_script || ''"
                @input="update({ dev_server_script: ($event.target as HTMLTextAreaElement).value })"
                :placeholder="t('settings.repos.devServerPlaceholder')"
                class="input min-h-[80px] font-mono text-sm"
              />
              <p class="text-xs text-text-low mt-1">
                {{ t('settings.repos.devServerHint') }}
              </p>
            </div>

            <!-- Setup 脚本 -->
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.repos.setup') }}
              </label>
              <textarea
                :value="draft.setup_script || ''"
                @input="update({ setup_script: ($event.target as HTMLTextAreaElement).value })"
                :placeholder="t('settings.repos.setupPlaceholder')"
                class="input min-h-[80px] font-mono text-sm"
              />
              <p class="text-xs text-text-low mt-1">
                {{ t('settings.repos.setupHint') }}
              </p>
              <label class="flex items-center gap-2 mt-2 cursor-pointer">
                <input
                  type="checkbox"
                  :checked="draft.parallel_setup_script ?? false"
                  @change="update({ parallel_setup_script: ($event.target as HTMLInputElement).checked })"
                  class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
                />
                <span class="text-sm text-text-primary">
                  {{ t('settings.repos.parallelSetup') }}
                </span>
                <span class="text-xs text-text-low">
                  - {{ t('settings.repos.parallelSetupHint') }}
                </span>
              </label>
            </div>

            <!-- Cleanup 脚本 -->
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.repos.cleanup') }}
              </label>
              <textarea
                :value="draft.cleanup_script || ''"
                @input="update({ cleanup_script: ($event.target as HTMLTextAreaElement).value })"
                :placeholder="t('settings.repos.cleanupPlaceholder')"
                class="input min-h-[80px] font-mono text-sm"
              />
              <p class="text-xs text-text-low mt-1">
                {{ t('settings.repos.cleanupHint') }}
              </p>
            </div>

            <!-- 复制文件 -->
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.repos.copyFiles') }}
              </label>
              <textarea
                :value="draft.copy_files || ''"
                @input="update({ copy_files: ($event.target as HTMLTextAreaElement).value })"
                :placeholder="t('settings.repos.copyFilesPlaceholder')"
                class="input min-h-[80px] font-mono text-sm"
              />
              <p class="text-xs text-text-low mt-1">
                {{ t('settings.repos.copyFilesHint') }}
              </p>
            </div>

            <!-- OS 相关占位符提示 -->
            <div class="p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
              <div class="flex items-start gap-2">
                <Info class="w-4 h-4 text-blue-500 flex-shrink-0 mt-0.5" />
                <div class="text-sm">
                  <div class="font-medium text-blue-800 dark:text-blue-200 mb-1">
                    {{ t('settings.repos.osPlaceholder.title') || '占位符提示' }}
                  </div>
                  <div class="text-blue-700 dark:text-blue-300 space-y-1">
                    <p>当前系统: <code class="font-mono bg-blue-100 dark:bg-blue-800 px-1 rounded">{{ osName }}</code></p>
                    <p>可用占位符:</p>
                    <ul class="list-disc list-inside ml-2 space-y-0.5">
                      <li><code class="font-mono bg-blue-100 dark:bg-blue-800 px-1 rounded">${HOME}</code> - 用户主目录</li>
                      <li><code class="font-mono bg-blue-100 dark:bg-blue-800 px-1 rounded">${REPO_ROOT}</code> - 仓库根目录</li>
                      <li><code class="font-mono bg-blue-100 dark:bg-blue-800 px-1 rounded">${WORKSPACE}</code> - 工作区目录</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Card>

        <!-- 保存底部栏 -->
        <SettingsSaveFooter
          :has-changes="hasChanges"
          :saving="saving"
          :success="success"
          :error="error"
          @save="handleSave"
          @discard="reset"
        />
      </template>
    </template>
  </div>
</template>
