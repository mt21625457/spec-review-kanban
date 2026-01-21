<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Card, Button, Input, Select, Loading, EditorAvailabilityIndicator, TagManager } from '@/components/ui'
import { SettingsSaveFooter } from '@/components/settings'
import { useLocalDraft } from '@/composables/useSettingsDraft'
import {
  configApi,
  systemApi,
  tagsApi,
  type AllSettings,
  type GiteaSettings,
  type ReviewSettings,
  type QueueSettings,
  type ConnectionTestResult,
} from '@/lib/api'
import type { Tag as ApiTag, CreateTag, UpdateTag } from '@/types'
import type { UserSystemInfo, ThemeMode, UiLanguage, EditorType, SoundFile } from '@/types/settings'
import { THEME_OPTIONS, LANGUAGE_OPTIONS, EDITOR_TYPES, SSH_SUPPORTED_EDITORS } from '@/types/settings'

// TagManager 需要的标签类型
interface SettingsTag {
  id: string
  name: string
  color?: string
}

const { t } = useI18n()

// ================== Vibe-Kanban 配置 ==================
const vibeSystemInfo = ref<UserSystemInfo | null>(null)
const loadingVibe = ref(true)

// 草稿管理（用于 Vibe-Kanban 配置）
const { draft, hasChanges, saving, success, error, init, update, reset, save } = useLocalDraft<{
  theme_mode: ThemeMode
  ui_language: UiLanguage
  editor: EditorType
  custom_command: string
  remote_ssh_host: string
  remote_ssh_user: string
  git_branch_prefix: string
  pr_auto_description: boolean
  pr_description_prompt: string
  sound_enabled: boolean
  sound_file: SoundFile
  analytics_enabled: boolean
  workspace_enabled: boolean
  commit_reminder_enabled: boolean
}>()

// 主题选项（带翻译）
const themeOptions = computed(() =>
  THEME_OPTIONS.map(opt => ({ value: opt.value, label: t(opt.labelKey) }))
)

// 语言选项
const languageOptions = LANGUAGE_OPTIONS

// 编辑器选项
const editorOptions = EDITOR_TYPES

// 声音选项
const soundOptions = [
  { value: 'default', label: 'Default' },
  { value: 'chime', label: 'Chime' },
  { value: 'bell', label: 'Bell' },
  { value: 'notification', label: 'Notification' },
  { value: 'none', label: 'None' },
]

// 是否显示自定义命令输入
const showCustomCommand = computed(() => draft.value?.editor === 'custom')

// 是否显示 SSH 配置
const showSshConfig = computed(() =>
  draft.value?.editor && SSH_SUPPORTED_EDITORS.includes(draft.value.editor)
)

// 分支前缀验证
const branchPrefixError = computed(() => {
  const prefix = draft.value?.git_branch_prefix || ''
  if (prefix.includes('/')) return t('settings.general.git.errors.slash')
  if (prefix.startsWith('.')) return t('settings.general.git.errors.startsWithDot')
  if (prefix.includes('..')) return t('settings.general.git.errors.doubleDot')
  if (/[@{}\[\]\\^~:?*]/.test(prefix)) return t('settings.general.git.errors.invalid')
  return ''
})

// 分支名称预览
const branchPreview = computed(() => {
  const prefix = draft.value?.git_branch_prefix || ''
  return `${prefix}task-123`
})

// 默认 PR 描述提示词模板
const defaultPrPrompt = `Generate a concise PR description based on the changes:

## Summary
- Briefly describe what this PR does
- List the main changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- Describe how this was tested`

// ================== 标签管理 ==================
const tags = ref<SettingsTag[]>([])
const loadingTags = ref(false)

// 将 API 标签转换为 UI 标签
const apiTagToSettingsTag = (apiTag: ApiTag): SettingsTag => ({
  id: apiTag.id,
  name: apiTag.tag_name,
  color: undefined, // API 不支持颜色，使用默认颜色
})

// 加载标签
const loadTags = async () => {
  loadingTags.value = true
  try {
    const data = await tagsApi.list()
    tags.value = data.map(apiTagToSettingsTag)
  } catch (err) {
    console.error('Failed to load tags:', err)
  } finally {
    loadingTags.value = false
  }
}

// 创建标签
const handleCreateTag = async (tag: Omit<SettingsTag, 'id'>) => {
  try {
    const createData: CreateTag = {
      tag_name: tag.name,
      content: '', // 默认空内容
    }
    const newTag = await tagsApi.create(createData)
    // 替换临时标签为实际标签
    const index = tags.value.findIndex(t => t.id.startsWith('temp-'))
    if (index !== -1) {
      tags.value[index] = apiTagToSettingsTag(newTag)
    }
  } catch (err) {
    console.error('Failed to create tag:', err)
    // 移除临时标签
    tags.value = tags.value.filter(t => !t.id.startsWith('temp-'))
  }
}

// 更新标签
const handleUpdateTag = async (tag: SettingsTag) => {
  try {
    const updateData: UpdateTag = {
      tag_name: tag.name,
    }
    await tagsApi.update(tag.id, updateData)
  } catch (err) {
    console.error('Failed to update tag:', err)
    // 重新加载以恢复
    await loadTags()
  }
}

// 删除标签
const handleDeleteTag = async (tagId: string) => {
  try {
    await tagsApi.delete(tagId)
  } catch (err) {
    console.error('Failed to delete tag:', err)
    // 重新加载以恢复
    await loadTags()
  }
}

// ================== AICodex 本地配置 ==================
const aicodexSettings = ref<AllSettings | null>(null)
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

// 加载状态
const loadingAicodex = ref(true)
const savingGitea = ref(false)
const savingReview = ref(false)
const savingQueue = ref(false)
const testResult = ref<ConnectionTestResult | null>(null)
const testLoading = ref(false)

// 代理选项
const agentOptions = [
  { value: 'codex', label: 'Codex' },
  { value: 'claude_code', label: 'Claude Code' },
  { value: 'gemini', label: 'Gemini' },
  { value: 'open_code', label: 'OpenCode' },
  { value: 'copilot', label: 'Copilot' },
]

// 加载 Vibe-Kanban 配置
const loadVibeSettings = async () => {
  loadingVibe.value = true
  try {
    const data = await systemApi.getInfo() as unknown as UserSystemInfo
    vibeSystemInfo.value = data
    if (data.config) {
      init({
        theme_mode: data.config.theme_mode || 'system',
        ui_language: data.config.ui_language || 'browser',
        editor: data.config.editor?.editor || 'vscode',
        custom_command: data.config.editor?.custom_command || '',
        remote_ssh_host: data.config.editor?.remote_ssh_host || '',
        remote_ssh_user: data.config.editor?.remote_ssh_user || '',
        git_branch_prefix: data.config.git_branch_prefix || '',
        pr_auto_description: data.config.pr_auto_description ?? true,
        pr_description_prompt: data.config.pr_description_prompt || '',
        sound_enabled: data.config.sound_enabled ?? true,
        sound_file: data.config.sound_file || 'default',
        analytics_enabled: data.config.analytics_enabled ?? true,
        workspace_enabled: data.config.workspace_enabled ?? false,
        commit_reminder_enabled: data.config.commit_reminder_enabled ?? false,
      })
    }
  } catch (err) {
    console.error('Failed to load Vibe-Kanban settings:', err)
  } finally {
    loadingVibe.value = false
  }
}

// 保存 Vibe-Kanban 配置
const handleSaveVibe = async () => {
  if (!draft.value) return

  await save(async (data) => {
    await systemApi.updateConfig({
      theme_mode: data.theme_mode,
      ui_language: data.ui_language,
      git_branch_prefix: data.git_branch_prefix,
      pr_auto_description: data.pr_auto_description,
      pr_description_prompt: data.pr_description_prompt,
      sound_enabled: data.sound_enabled,
      sound_file: data.sound_file,
      analytics_enabled: data.analytics_enabled,
      workspace_enabled: data.workspace_enabled,
      commit_reminder_enabled: data.commit_reminder_enabled,
      editor: {
        editor: data.editor,
        custom_command: data.custom_command || undefined,
        remote_ssh_host: data.remote_ssh_host || undefined,
        remote_ssh_user: data.remote_ssh_user || undefined,
      },
    })
  })
}

// 加载 AICodex 配置
const loadAicodexSettings = async () => {
  loadingAicodex.value = true
  try {
    const response = await configApi.getAll()
    if (response.success && response.data) {
      aicodexSettings.value = response.data
      giteaForm.value = { ...response.data.gitea }
      reviewForm.value = { ...response.data.review }
      queueForm.value = { ...response.data.queue }
    }
  } catch (err) {
    console.error('Failed to load AICodex settings:', err)
  } finally {
    loadingAicodex.value = false
  }
}

// 保存 Gitea 配置
const saveGiteaSettings = async () => {
  savingGitea.value = true
  try {
    const response = await configApi.updateGitea(giteaForm.value)
    if (response.success) {
      alert(t('common.success'))
    } else {
      alert(response.error || t('common.error'))
    }
  } finally {
    savingGitea.value = false
  }
}

// 测试 Gitea 连接
const testGiteaConnection = async () => {
  testLoading.value = true
  testResult.value = null
  try {
    const response = await configApi.testGiteaConnection()
    if (response.data) {
      testResult.value = response.data
    }
  } finally {
    testLoading.value = false
  }
}

// 保存审核配置
const saveReviewSettings = async () => {
  savingReview.value = true
  try {
    const response = await configApi.updateReview(reviewForm.value)
    if (response.success) {
      alert(t('common.success'))
    } else {
      alert(response.error || t('common.error'))
    }
  } finally {
    savingReview.value = false
  }
}

// 保存队列配置
const saveQueueSettings = async () => {
  savingQueue.value = true
  try {
    const response = await configApi.updateQueue(queueForm.value)
    if (response.success) {
      alert(t('common.success'))
    } else {
      alert(response.error || t('common.error'))
    }
  } finally {
    savingQueue.value = false
  }
}

// 重置免责声明
const resetDisclaimer = async () => {
  if (!confirm(t('settings.general.security.resetDisclaimerConfirm'))) return
  try {
    await systemApi.updateConfig({ disclaimer_acknowledged: false })
    alert(t('common.success'))
  } catch {
    alert(t('common.error'))
  }
}

// 重置引导
const resetOnboarding = async () => {
  if (!confirm(t('settings.general.security.resetOnboardingConfirm'))) return
  try {
    await systemApi.updateConfig({ onboarding_acknowledged: false })
    alert(t('common.success'))
  } catch {
    alert(t('common.error'))
  }
}

// 加载配置
onMounted(async () => {
  await Promise.all([loadVibeSettings(), loadAicodexSettings(), loadTags()])
})
</script>

<template>
  <div class="space-y-6">
    <h2 class="text-xl font-semibold text-text-primary">
      {{ t('settings.general.title') }}
    </h2>

    <div v-if="loadingVibe && loadingAicodex" class="py-12">
      <Loading />
    </div>

    <template v-else>
      <!-- ================== Vibe-Kanban 配置 ================== -->

      <!-- 外观设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.appearance.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <!-- 主题 -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.appearance.theme') }}
            </label>
            <Select
              :model-value="draft?.theme_mode"
              :options="themeOptions"
              :placeholder="t('settings.general.appearance.themePlaceholder')"
              @update:model-value="update({ theme_mode: $event as ThemeMode })"
            />
          </div>

          <!-- 语言 -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.appearance.language') }}
            </label>
            <Select
              :model-value="draft?.ui_language"
              :options="languageOptions"
              :placeholder="t('settings.general.appearance.languagePlaceholder')"
              @update:model-value="update({ ui_language: $event as UiLanguage })"
            />
          </div>
        </div>
      </Card>

      <!-- 编辑器设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.editor.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <!-- 编辑器类型 -->
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.editor.type') }}
            </label>
            <div class="flex items-center gap-3">
              <Select
                :model-value="draft?.editor"
                :options="editorOptions"
                :placeholder="t('settings.general.editor.typePlaceholder')"
                class="flex-1"
                @update:model-value="update({ editor: $event as EditorType })"
              />
              <EditorAvailabilityIndicator
                v-if="draft?.editor"
                :editor-type="draft.editor"
                :custom-command="draft.custom_command"
              />
            </div>
          </div>

          <!-- 自定义命令 -->
          <div v-if="showCustomCommand">
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.editor.customCommand') }}
            </label>
            <Input
              :model-value="draft?.custom_command || ''"
              :placeholder="t('settings.general.editor.customCommandPlaceholder')"
              @update:model-value="update({ custom_command: $event as string })"
            />
          </div>

          <!-- SSH 配置 -->
          <template v-if="showSshConfig">
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.general.editor.remoteSSH.host') }}
              </label>
              <Input
                :model-value="draft?.remote_ssh_host || ''"
                :placeholder="t('settings.general.editor.remoteSSH.hostPlaceholder')"
                @update:model-value="update({ remote_ssh_host: $event as string })"
              />
            </div>
            <div v-if="draft?.remote_ssh_host">
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.general.editor.remoteSSH.user') }}
              </label>
              <Input
                :model-value="draft?.remote_ssh_user || ''"
                :placeholder="t('settings.general.editor.remoteSSH.userPlaceholder')"
                @update:model-value="update({ remote_ssh_user: $event as string })"
              />
            </div>
          </template>
        </div>
      </Card>

      <!-- Git 设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.git.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.git.branchPrefix') }}
            </label>
            <Input
              :model-value="draft?.git_branch_prefix || ''"
              :placeholder="t('settings.general.git.branchPrefixPlaceholder')"
              @update:model-value="update({ git_branch_prefix: $event as string })"
            />
            <p v-if="branchPrefixError" class="text-sm text-red-500 mt-1">
              {{ branchPrefixError }}
            </p>
            <p v-else class="text-xs text-text-low mt-1">
              {{ t('settings.general.git.branchPreview') }}: <code class="font-mono">{{ branchPreview }}</code>
            </p>
          </div>
        </div>
      </Card>

      <!-- Pull Request 设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.pr.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              :checked="draft?.pr_auto_description"
              @change="update({ pr_auto_description: ($event.target as HTMLInputElement).checked })"
              class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">
              {{ t('settings.general.pr.autoDescription') }}
            </span>
          </label>
          <p class="text-xs text-text-low -mt-2 ml-6">
            {{ t('settings.general.pr.autoDescriptionHint') }}
          </p>

          <div v-if="draft?.pr_auto_description">
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.pr.customPrompt') }}
            </label>
            <textarea
              :value="draft?.pr_description_prompt || ''"
              @input="update({ pr_description_prompt: ($event.target as HTMLTextAreaElement).value })"
              :placeholder="defaultPrPrompt"
              class="input min-h-[100px] font-mono text-sm"
            />
            <p class="text-xs text-text-low mt-1">
              {{ t('settings.general.pr.customPromptHint') }}
            </p>
          </div>
        </div>
      </Card>

      <!-- 隐私设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.privacy.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              :checked="draft?.analytics_enabled"
              @change="update({ analytics_enabled: ($event.target as HTMLInputElement).checked })"
              class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">
              {{ t('settings.general.privacy.analytics') }}
            </span>
          </label>
          <p class="text-xs text-text-low -mt-2 ml-6">
            {{ t('settings.general.privacy.analyticsHint') }}
          </p>
        </div>
      </Card>

      <!-- 通知设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.notifications.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              :checked="draft?.sound_enabled"
              @change="update({ sound_enabled: ($event.target as HTMLInputElement).checked })"
              class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">
              {{ t('settings.general.notifications.sound') }}
            </span>
          </label>
          <p class="text-xs text-text-low -mt-2 ml-6">
            {{ t('settings.general.notifications.soundHint') }}
          </p>

          <div v-if="draft?.sound_enabled">
            <label class="block text-sm font-medium text-text-primary mb-1">
              {{ t('settings.general.notifications.soundFile') }}
            </label>
            <Select
              :model-value="draft?.sound_file"
              :options="soundOptions"
              :placeholder="t('settings.general.notifications.soundFilePlaceholder')"
              @update:model-value="update({ sound_file: $event as SoundFile })"
            />
          </div>
        </div>
      </Card>

      <!-- 任务模板标签 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.templates.title') }}
          </h3>
        </template>

        <div v-if="loadingTags" class="py-8">
          <Loading />
        </div>
        <div v-else>
          <p class="text-sm text-text-low mb-4">
            {{ t('settings.general.templates.description') }}
          </p>
          <TagManager
            v-model="tags"
            :placeholder="t('settings.general.templates.empty')"
            @create="handleCreateTag"
            @update="handleUpdateTag"
            @delete="handleDeleteTag"
          />
        </div>
      </Card>

      <!-- Beta 功能 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.beta.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              :checked="draft?.workspace_enabled"
              @change="update({ workspace_enabled: ($event.target as HTMLInputElement).checked })"
              class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">
              {{ t('settings.general.beta.workspace') }}
            </span>
          </label>
          <p class="text-xs text-text-low -mt-2 ml-6">
            {{ t('settings.general.beta.workspaceHint') }}
          </p>

          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              :checked="draft?.commit_reminder_enabled"
              @change="update({ commit_reminder_enabled: ($event.target as HTMLInputElement).checked })"
              class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">
              {{ t('settings.general.beta.commitReminder') }}
            </span>
          </label>
          <p class="text-xs text-text-low -mt-2 ml-6">
            {{ t('settings.general.beta.commitReminderHint') }}
          </p>
        </div>
      </Card>

      <!-- 安全设置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.general.security.title') }}
          </h3>
        </template>

        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm font-medium text-text-primary">
                {{ t('settings.general.security.resetDisclaimer') }}
              </div>
              <div class="text-xs text-text-low">
                {{ t('settings.general.security.resetDisclaimerHint') }}
              </div>
            </div>
            <Button variant="secondary" size="sm" @click="resetDisclaimer">
              {{ t('common.reset') }}
            </Button>
          </div>

          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm font-medium text-text-primary">
                {{ t('settings.general.security.resetOnboarding') }}
              </div>
              <div class="text-xs text-text-low">
                {{ t('settings.general.security.resetOnboardingHint') }}
              </div>
            </div>
            <Button variant="secondary" size="sm" @click="resetOnboarding">
              {{ t('common.reset') }}
            </Button>
          </div>
        </div>
      </Card>

      <!-- Vibe-Kanban 保存底部栏 -->
      <SettingsSaveFooter
        :has-changes="hasChanges"
        :saving="saving"
        :success="success"
        :error="error"
        @save="handleSaveVibe"
        @discard="reset"
      />

      <!-- ================== AICodex 本地配置分隔线 ================== -->
      <div class="border-t border-border-normal pt-6 mt-8">
        <h3 class="text-lg font-semibold text-text-primary mb-4">
          AICodex 本地配置
        </h3>
      </div>

      <!-- Gitea 配置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.gitea.title') }}
          </h3>
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
            <Button variant="primary" type="submit" :loading="savingGitea">
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
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.review.title') }}
          </h3>
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
              <span class="text-sm text-text-primary">
                {{ t('settings.review.autoStart') }}
              </span>
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
            <Button variant="primary" type="submit" :loading="savingReview">
              {{ t('common.save') }}
            </Button>
          </div>
        </form>
      </Card>

      <!-- 队列配置 -->
      <Card>
        <template #header>
          <h3 class="font-semibold text-text-primary">
            {{ t('settings.queue.title') }}
          </h3>
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
            <Button variant="primary" type="submit" :loading="savingQueue">
              {{ t('common.save') }}
            </Button>
          </div>
        </form>
      </Card>
    </template>
  </div>
</template>
