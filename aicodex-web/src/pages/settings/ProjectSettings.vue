<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Card, Button, Input, Select, Loading } from '@/components/ui'
import { SettingsSaveFooter, RepoPickerDialog } from '@/components/settings'
import { useLocalDraft } from '@/composables/useSettingsDraft'
import { projectsApi } from '@/lib/api'
import type { Project, Repo } from '@/types'
import { Plus, Trash2, ExternalLink } from 'lucide-vue-next'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

// 项目列表和选中项目
const projects = ref<Project[]>([])
const selectedProjectId = ref<string | null>(null)
const projectRepos = ref<Repo[]>([])

// 草稿管理
const { draft, hasChanges, saving, success, error, init, update, reset, save } = useLocalDraft<{
  name: string
}>()

// 加载状态
const loading = ref(true)
const loadingRepos = ref(false)

// 计算当前选中的项目
const selectedProject = computed(() =>
  projects.value.find(p => p.id === selectedProjectId.value) || null
)

// 项目选项
const projectOptions = computed(() =>
  projects.value.map(p => ({
    value: p.id,
    label: p.name
  }))
)

// 加载项目列表
onMounted(async () => {
  await loadProjects()
})

// 监听 URL 参数变化
watch(() => route.query.projectId, (newId) => {
  if (newId && typeof newId === 'string') {
    selectedProjectId.value = newId
  }
}, { immediate: true })

// 监听选中项目变化
watch(selectedProjectId, async (newId) => {
  if (newId) {
    router.replace({ query: { ...route.query, projectId: newId } })
    await loadProjectRepos(newId)
    const project = projects.value.find(p => p.id === newId)
    if (project) {
      init({ name: project.name })
    }
  }
})

const loadProjects = async () => {
  loading.value = true
  try {
    const data = await projectsApi.list()
    projects.value = data
    // 如果 URL 没有指定项目，选择第一个
    if (!selectedProjectId.value && data.length > 0) {
      selectedProjectId.value = data[0].id
    }
  } catch (err) {
    console.error('Failed to load projects:', err)
  } finally {
    loading.value = false
  }
}

const loadProjectRepos = async (projectId: string) => {
  loadingRepos.value = true
  try {
    const data = await projectsApi.getRepositories(projectId)
    projectRepos.value = data
  } catch (err) {
    console.error('Failed to load project repos:', err)
    projectRepos.value = []
  } finally {
    loadingRepos.value = false
  }
}

// 保存项目
const handleSave = async () => {
  if (!selectedProjectId.value || !draft.value) return

  await save(async (data) => {
    await projectsApi.update(selectedProjectId.value!, { name: data.name })
    // 更新本地列表
    const index = projects.value.findIndex(p => p.id === selectedProjectId.value)
    if (index !== -1) {
      projects.value[index] = { ...projects.value[index], name: data.name }
    }
  })
}

// 删除项目关联的仓库
const removeRepo = async (repoId: string) => {
  if (!selectedProjectId.value) return
  if (!confirm(t('settings.projects.removeRepo') + '?')) return

  try {
    await projectsApi.deleteRepository(selectedProjectId.value, repoId)
    projectRepos.value = projectRepos.value.filter(r => r.id !== repoId)
  } catch (err) {
    console.error('Failed to remove repo:', err)
  }
}

// 跳转到仓库设置
const goToRepoSettings = (repoId: string) => {
  router.push({ path: '/settings/repos', query: { repoId } })
}

// 仓库选择对话框
const showRepoPickerDialog = ref(false)
const addingRepo = ref(false)

// 添加仓库
const handleAddRepo = async (repo: { path: string; displayName: string }) => {
  if (!selectedProjectId.value) return

  addingRepo.value = true
  try {
    const newRepo = await projectsApi.addRepository(selectedProjectId.value, {
      git_repo_path: repo.path,
      display_name: repo.displayName,
    })
    projectRepos.value.push(newRepo)
  } catch (err) {
    console.error('Failed to add repo:', err)
    alert(t('common.error'))
  } finally {
    addingRepo.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-text-primary">
        {{ t('settings.projects.title') }}
      </h2>
    </div>

    <div v-if="loading" class="py-12">
      <Loading />
    </div>

    <template v-else>
      <!-- 项目选择器 -->
      <Card>
        <div class="flex items-center gap-4">
          <label class="text-sm font-medium text-text-primary whitespace-nowrap">
            {{ t('settings.projects.select') }}
          </label>
          <Select
            v-model="selectedProjectId"
            :options="projectOptions"
            :placeholder="t('settings.projects.selectPlaceholder')"
            class="flex-1"
          />
        </div>
      </Card>

      <template v-if="selectedProject">
        <!-- 项目常规设置 -->
        <Card>
          <template #header>
            <h3 class="font-semibold text-text-primary">
              {{ t('settings.projects.general') }}
            </h3>
          </template>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-text-primary mb-1">
                {{ t('settings.projects.name') }}
              </label>
              <Input
                :model-value="draft?.name || ''"
                @update:model-value="update({ name: $event as string })"
                :placeholder="t('settings.projects.namePlaceholder')"
              />
            </div>
          </div>
        </Card>

        <!-- 仓库管理 -->
        <Card>
          <template #header>
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-text-primary">
                  {{ t('settings.projects.repositories') }}
                </h3>
                <p class="text-sm text-text-low mt-0.5">
                  {{ t('settings.projects.repositoriesDesc') }}
                </p>
              </div>
              <Button variant="secondary" size="sm" :loading="addingRepo" @click="showRepoPickerDialog = true">
                <Plus class="w-4 h-4 mr-1" />
                {{ t('settings.projects.addRepo') }}
              </Button>
            </div>
          </template>

          <div v-if="loadingRepos" class="py-8">
            <Loading />
          </div>
          <div v-else-if="projectRepos.length === 0" class="py-8 text-center text-text-low">
            {{ t('settings.projects.noRepos') }}
          </div>
          <div v-else class="divide-y divide-border-normal -mx-4">
            <div
              v-for="repo in projectRepos"
              :key="repo.id"
              class="flex items-center justify-between px-4 py-3 hover:bg-bg-hover transition-colors"
            >
              <div class="min-w-0 flex-1">
                <div class="font-medium text-text-primary truncate">
                  {{ repo.display_name || repo.name }}
                </div>
                <div class="text-sm text-text-low font-mono truncate">
                  {{ repo.path }}
                </div>
              </div>
              <div class="flex items-center gap-2 ml-4">
                <Button
                  variant="ghost"
                  size="sm"
                  @click="goToRepoSettings(repo.id)"
                >
                  <ExternalLink class="w-4 h-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  @click="removeRepo(repo.id)"
                >
                  <Trash2 class="w-4 h-4 text-red-500" />
                </Button>
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

    <!-- 仓库选择对话框 -->
    <RepoPickerDialog
      v-model:open="showRepoPickerDialog"
      @select="handleAddRepo"
    />
  </div>
</template>
