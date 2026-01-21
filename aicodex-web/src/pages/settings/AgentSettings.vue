<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Card, Button, Select, Loading, Dialog, Input, ExecutorConfigForm, JsonEditor } from '@/components/ui'
import { SettingsSaveFooter } from '@/components/settings'
import { useLocalDraft } from '@/composables/useSettingsDraft'
import { systemApi, profilesApi, type ProfilesConfig } from '@/lib/api'
import type { BaseCodingAgent, AvailabilityInfo } from '@/types'
import type { UserSystemInfo, ExecutorConfig } from '@/types/settings'
import { Check, X, Loader2, Plus, Trash2, AlertTriangle } from 'lucide-vue-next'

const { t } = useI18n()

// 系统信息
const systemInfo = ref<UserSystemInfo | null>(null)
const profilesConfig = ref<ProfilesConfig | null>(null)

// 执行器配置
const selectedExecutor = ref<string>('claude-code')
const selectedConfig = ref<string>('default')
const useFormEditor = ref(true)

// 可用性状态
const availability = ref<AvailabilityInfo | null>(null)
const checkingAvailability = ref(false)

// 草稿管理
const { draft, hasChanges, saving, success, error, init, update, reset, save } = useLocalDraft<{
  executor_profile: string
  config: ExecutorConfig
}>()

// 加载状态
const loading = ref(true)
const savingProfiles = ref(false)

// 对话框状态
const showCreateDialog = ref(false)
const showDeleteDialog = ref(false)
const newConfigName = ref('')
const createError = ref('')

// 执行器选项
const executorOptions = computed(() => {
  if (!systemInfo.value?.profiles?.executors) return []
  return Object.keys(systemInfo.value.profiles.executors).map(key => ({
    value: key,
    label: key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
  }))
})

// 配置选项（包含创建新配置选项）
const configOptions = computed(() => {
  const options = [
    { value: 'default', label: 'Default' },
    { value: 'fast', label: 'Fast' },
    { value: 'thorough', label: 'Thorough' },
  ]
  return options
})

// 当前执行器配置
const currentConfig = computed<ExecutorConfig>(() => {
  if (!profilesConfig.value?.executors?.[selectedExecutor.value]) {
    return {}
  }
  return profilesConfig.value.executors[selectedExecutor.value] as ExecutorConfig
})

// 配置的 JSON 字符串
const configJsonString = computed(() => {
  return JSON.stringify(currentConfig.value, null, 2)
})

// 是否可以删除配置
const canDeleteConfig = computed(() => {
  return selectedConfig.value !== 'default'
})

// 加载系统信息
onMounted(async () => {
  await loadSystemInfo()
  await loadProfiles()
})

// 监听执行器变化，检查可用性
watch(selectedExecutor, async (newExecutor) => {
  if (newExecutor) {
    await checkAvailability(newExecutor as BaseCodingAgent)
  }
})

const loadSystemInfo = async () => {
  loading.value = true
  try {
    const data = await systemApi.getInfo() as unknown as UserSystemInfo
    systemInfo.value = data
    if (data.config?.executor_profile) {
      selectedExecutor.value = data.config.executor_profile
      init({
        executor_profile: data.config.executor_profile,
        config: {},
      })
    }
    // 检查当前执行器可用性
    if (selectedExecutor.value) {
      await checkAvailability(selectedExecutor.value as BaseCodingAgent)
    }
  } catch (err) {
    console.error('Failed to load system info:', err)
  } finally {
    loading.value = false
  }
}

const loadProfiles = async () => {
  try {
    const data = await profilesApi.get()
    profilesConfig.value = data
  } catch (err) {
    console.error('Failed to load profiles:', err)
  }
}

const checkAvailability = async (executor: BaseCodingAgent) => {
  checkingAvailability.value = true
  availability.value = null
  try {
    const result = await systemApi.checkAgentAvailability(executor)
    availability.value = result
  } catch (err) {
    availability.value = { available: false, message: '检查失败' }
  } finally {
    checkingAvailability.value = false
  }
}

// 保存执行器配置
const handleSave = async () => {
  if (!draft.value) return

  await save(async (data) => {
    await systemApi.updateConfig({ executor_profile: data.executor_profile })
  })
}

// 保存 profiles 配置
const handleSaveProfiles = async () => {
  if (!profilesConfig.value) return

  savingProfiles.value = true
  try {
    await profilesApi.update(profilesConfig.value)
    alert(t('common.success'))
  } catch (err) {
    console.error('Failed to save profiles:', err)
    alert(t('common.error'))
  } finally {
    savingProfiles.value = false
  }
}

// 更新当前执行器配置
const handleConfigUpdate = (newConfig: ExecutorConfig) => {
  if (profilesConfig.value && selectedExecutor.value) {
    profilesConfig.value.executors[selectedExecutor.value] = newConfig
  }
}

// 更新 JSON 配置
const handleJsonChange = (json: string) => {
  try {
    const parsed = JSON.parse(json)
    handleConfigUpdate(parsed)
  } catch {
    // JSON 解析失败，忽略
  }
}

// 打开创建配置对话框
const openCreateDialog = () => {
  newConfigName.value = ''
  createError.value = ''
  showCreateDialog.value = true
}

// 创建新配置
const handleCreateConfig = () => {
  const name = newConfigName.value.trim().toLowerCase().replace(/\s+/g, '-')
  if (!name) {
    createError.value = t('settings.agents.config.errors.nameRequired')
    return
  }
  if (configOptions.value.some(opt => opt.value === name)) {
    createError.value = t('settings.agents.config.errors.nameExists')
    return
  }

  // 添加新配置选项
  selectedConfig.value = name
  showCreateDialog.value = false
}

// 确认删除配置
const confirmDeleteConfig = () => {
  if (!canDeleteConfig.value) return
  showDeleteDialog.value = true
}

// 删除配置
const handleDeleteConfig = () => {
  // 切换回默认配置
  selectedConfig.value = 'default'
  showDeleteDialog.value = false
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-text-primary">
        {{ t('settings.agents.title') }}
      </h2>
    </div>

    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <template v-else>
      <!-- 任务执行配置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.agents.taskExecution') }}
          </h3>
        </template>

        <div class="space-y-4">
          <!-- 执行器选择 -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.agents.executor') }}
            </label>
            <div class="flex items-center gap-3">
              <Select
                v-model="selectedExecutor"
                :options="executorOptions"
                :placeholder="t('settings.agents.executorPlaceholder')"
                class="flex-1"
                @update:model-value="update({ executor_profile: $event as string })"
              />
              <!-- 可用性指示器 -->
              <div class="flex items-center gap-1.5">
                <template v-if="checkingAvailability">
                  <Loader2 class="w-4 h-4 animate-spin text-text-low" />
                  <span class="text-sm text-text-low">
                    {{ t('settings.agents.availability.checking') }}
                  </span>
                </template>
                <template v-else-if="availability">
                  <template v-if="availability.available">
                    <Check class="w-4 h-4 text-green-500" />
                    <span class="text-sm text-green-600 dark:text-green-400">
                      {{ t('settings.agents.availability.available') }}
                    </span>
                  </template>
                  <template v-else>
                    <X class="w-4 h-4 text-red-500" />
                    <span class="text-sm text-red-600 dark:text-red-400">
                      {{ t('settings.agents.availability.unavailable') }}
                    </span>
                  </template>
                </template>
              </div>
            </div>
          </div>

          <!-- 配置选择器 -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.agents.config.select') }}
            </label>
            <div class="flex items-center gap-2">
              <Select
                v-model="selectedConfig"
                :options="configOptions"
                :placeholder="t('settings.agents.config.selectPlaceholder')"
                class="flex-1"
              />
              <Button variant="secondary" size="sm" @click="openCreateDialog">
                <Plus class="w-4 h-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                :disabled="!canDeleteConfig"
                @click="confirmDeleteConfig"
              >
                <Trash2 class="w-4 h-4 text-red-500" />
              </Button>
            </div>
          </div>
        </div>
      </Card>

      <!-- 代理配置编辑器 -->
      <Card>
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="font-semibold text-text-primary">
              {{ t('settings.agents.config.title') }}
            </h3>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="useFormEditor"
                type="checkbox"
                class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
              />
              <span class="text-sm text-text-normal">
                {{ t('settings.agents.config.useFormEditor') }}
              </span>
            </label>
          </div>
        </template>

        <div class="space-y-4">
          <!-- 执行器类型 -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.agents.config.type') }}
            </label>
            <Select
              v-model="selectedExecutor"
              :options="executorOptions"
              :placeholder="t('settings.agents.config.typePlaceholder')"
            />
          </div>

          <!-- 配置编辑区域 -->
          <div v-if="useFormEditor">
            <!-- 表单编辑器 -->
            <ExecutorConfigForm
              :executor-type="selectedExecutor"
              :model-value="currentConfig"
              @update:model-value="handleConfigUpdate"
            />
          </div>
          <div v-else>
            <!-- JSON 编辑器 -->
            <JsonEditor
              :model-value="configJsonString"
              :show-toolbar="true"
              min-height="250px"
              @update:model-value="handleJsonChange"
            />
          </div>

          <!-- 配置文件路径 -->
          <div v-if="systemInfo?.environment?.profiles_path">
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.agents.config.path') }}
            </label>
            <div class="px-3 py-2 bg-bg-secondary rounded-lg font-mono text-sm text-text-normal">
              {{ systemInfo.environment.profiles_path }}
            </div>
          </div>

          <!-- 保存 profiles 配置按钮 -->
          <div class="flex justify-end pt-2">
            <Button variant="primary" :loading="savingProfiles" @click="handleSaveProfiles">
              {{ t('settings.agents.config.saveProfiles') }}
            </Button>
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

    <!-- 创建配置对话框 -->
    <Dialog :open="showCreateDialog" @update:open="showCreateDialog = $event">
      <div class="w-[400px] p-6">
        <h3 class="text-lg font-semibold text-text-primary mb-4">
          {{ t('settings.agents.config.createTitle') }}
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.agents.config.name') }}
            </label>
            <Input
              v-model="newConfigName"
              :placeholder="t('settings.agents.config.namePlaceholder')"
              autofocus
              @keydown.enter="handleCreateConfig"
            />
            <p v-if="createError" class="text-sm text-red-500 mt-1">
              {{ createError }}
            </p>
          </div>
          <div class="flex justify-end gap-2">
            <Button variant="secondary" @click="showCreateDialog = false">
              {{ t('common.cancel') }}
            </Button>
            <Button variant="primary" @click="handleCreateConfig">
              {{ t('common.create') }}
            </Button>
          </div>
        </div>
      </div>
    </Dialog>

    <!-- 删除配置确认对话框 -->
    <Dialog :open="showDeleteDialog" @update:open="showDeleteDialog = $event">
      <div class="w-[400px] p-6">
        <div class="flex items-start gap-3 mb-4">
          <AlertTriangle class="w-6 h-6 text-amber-500 flex-shrink-0" />
          <div>
            <h3 class="text-lg font-semibold text-text-primary">
              {{ t('settings.agents.config.deleteTitle') }}
            </h3>
            <p class="text-sm text-text-low mt-1">
              {{ t('settings.agents.config.deleteConfirm') }}
            </p>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <Button variant="secondary" @click="showDeleteDialog = false">
            {{ t('common.cancel') }}
          </Button>
          <Button variant="primary" class="bg-red-500 hover:bg-red-600" @click="handleDeleteConfig">
            {{ t('common.delete') }}
          </Button>
        </div>
      </div>
    </Dialog>
  </div>
</template>
