<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { repoMappingApi, type RepoMapping, type CreateRepoMapping } from '@/lib/api'
import { PageHeader } from '@/components/layout'
import { Card, Button, Input, Select, Dialog, Loading } from '@/components/ui'

const { t } = useI18n()

const mappings = ref<RepoMapping[]>([])
const loading = ref(true)
const showDialog = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)

const form = ref<CreateRepoMapping>({
  gitea_repo: '',
  local_path: '',
  agent_type: 'codex',
  is_enabled: true,
})

const agentOptions = [
  { value: 'codex', label: 'Codex' },
  { value: 'claude_code', label: 'Claude Code' },
  { value: 'gemini', label: 'Gemini' },
  { value: 'open_code', label: 'OpenCode' },
  { value: 'copilot', label: 'Copilot' },
]

onMounted(async () => {
  await loadMappings()
})

const loadMappings = async () => {
  loading.value = true
  const response = await repoMappingApi.list()
  if (response.success && response.data) {
    mappings.value = response.data
  }
  loading.value = false
}

const openDialog = (mapping?: RepoMapping) => {
  if (mapping) {
    editingId.value = mapping.id
    form.value = {
      gitea_repo: mapping.gitea_repo,
      local_path: mapping.local_path,
      vibe_project_id: mapping.vibe_project_id,
      agent_type: mapping.agent_type,
      executor_profile_id: mapping.executor_profile_id,
      custom_prompt: mapping.custom_prompt,
      is_enabled: mapping.is_enabled,
    }
  } else {
    editingId.value = null
    form.value = {
      gitea_repo: '',
      local_path: '',
      agent_type: 'codex',
      is_enabled: true,
    }
  }
  showDialog.value = true
}

const closeDialog = () => {
  showDialog.value = false
  editingId.value = null
}

const handleSubmit = async () => {
  saving.value = true
  let response
  if (editingId.value) {
    response = await repoMappingApi.update(editingId.value, form.value)
  } else {
    response = await repoMappingApi.create(form.value)
  }
  if (response.success) {
    closeDialog()
    await loadMappings()
  } else {
    alert(response.error || t('common.error'))
  }
  saving.value = false
}

const handleDelete = async (id: string) => {
  if (!confirm(t('repoMappings.confirmDelete'))) return
  const response = await repoMappingApi.delete(id)
  if (response.success) {
    await loadMappings()
  } else {
    alert(response.error || t('common.error'))
  }
}

const toggleEnabled = async (mapping: RepoMapping) => {
  const response = await repoMappingApi.update(mapping.id, {
    is_enabled: !mapping.is_enabled,
  })
  if (response.success) {
    await loadMappings()
  }
}
</script>

<template>
  <div class="max-w-3xl">
    <PageHeader :title="t('repoMappings.title')">
      <template #actions>
        <Button variant="primary" @click="openDialog()">
          <svg class="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t('repoMappings.add') }}
        </Button>
      </template>
    </PageHeader>

    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <Card v-else-if="mappings.length === 0">
      <div class="py-12 text-center text-text-muted">
        {{ t('repoMappings.empty') }}
      </div>
    </Card>

    <div v-else class="space-y-4">
      <Card v-for="mapping in mappings" :key="mapping.id">
        <div class="flex items-start justify-between mb-3">
          <h3 class="font-semibold text-text-primary">{{ mapping.gitea_repo }}</h3>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              :checked="mapping.is_enabled"
              class="sr-only peer"
              @change="toggleEnabled(mapping)"
            />
            <div class="w-11 h-6 bg-bg-tertiary peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-border-normal after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-brand"></div>
          </label>
        </div>

        <div class="space-y-2 text-sm mb-4">
          <div class="flex gap-2">
            <span class="text-text-muted">{{ t('repoMappings.dialog.localPath') }}:</span>
            <span class="text-text-primary">{{ mapping.local_path }}</span>
          </div>
          <div class="flex gap-2">
            <span class="text-text-muted">{{ t('repoMappings.dialog.agentType') }}:</span>
            <span class="text-text-primary">{{ mapping.agent_type }}</span>
          </div>
          <div v-if="mapping.vibe_project_id" class="flex gap-2">
            <span class="text-text-muted">{{ t('repoMappings.dialog.vibeProjectId') }}:</span>
            <span class="text-text-primary">{{ mapping.vibe_project_id }}</span>
          </div>
        </div>

        <div class="flex gap-3 pt-3 border-t border-border-normal">
          <button
            class="text-sm text-brand hover:underline"
            @click="openDialog(mapping)"
          >
            {{ t('common.edit') }}
          </button>
          <button
            class="text-sm text-red-500 hover:underline"
            @click="handleDelete(mapping.id)"
          >
            {{ t('common.delete') }}
          </button>
        </div>
      </Card>
    </div>

    <!-- 对话框 -->
    <Dialog
      v-model:open="showDialog"
      :title="editingId ? t('repoMappings.dialog.editTitle') : t('repoMappings.dialog.createTitle')"
      size="md"
      @close="closeDialog"
    >
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-primary mb-1">
            {{ t('repoMappings.dialog.giteaRepo') }}
          </label>
          <Input
            v-model="form.gitea_repo"
            :placeholder="t('repoMappings.dialog.giteaRepoPlaceholder')"
            :disabled="!!editingId"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-primary mb-1">
            {{ t('repoMappings.dialog.localPath') }}
          </label>
          <Input v-model="form.local_path" placeholder="/path/to/repo" />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-primary mb-1">
            {{ t('repoMappings.dialog.vibeProjectId') }}
          </label>
          <Input v-model="form.vibe_project_id" placeholder="可选" />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-primary mb-1">
            {{ t('repoMappings.dialog.agentType') }}
          </label>
          <Select v-model="form.agent_type" :options="agentOptions" />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-primary mb-1">
            {{ t('repoMappings.dialog.executorProfileId') }}
          </label>
          <Input v-model="form.executor_profile_id" placeholder="可选" />
        </div>
        <div>
          <label class="block text-sm font-medium text-text-primary mb-1">
            {{ t('repoMappings.dialog.customPrompt') }}
          </label>
          <textarea
            v-model="form.custom_prompt"
            rows="4"
            class="input resize-none"
            placeholder="可选的自定义 Prompt"
          />
        </div>
        <div>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              v-model="form.is_enabled"
              type="checkbox"
              class="w-4 h-4 rounded border-border-normal text-brand focus:ring-brand"
            />
            <span class="text-sm text-text-primary">{{ t('repoMappings.dialog.enabled') }}</span>
          </label>
        </div>
      </form>

      <template #footer>
        <div class="flex justify-end gap-3">
          <Button variant="secondary" @click="closeDialog">
            {{ t('common.cancel') }}
          </Button>
          <Button variant="primary" :loading="saving" @click="handleSubmit">
            {{ t('common.save') }}
          </Button>
        </div>
      </template>
    </Dialog>
  </div>
</template>
