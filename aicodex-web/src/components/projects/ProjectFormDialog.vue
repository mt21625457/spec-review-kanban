<script setup lang="ts">
/**
 * ProjectFormDialog - Dialog for creating and editing projects
 * Create mode: First shows RepoPickerDialog, then creates project with selected repo
 * Edit mode: Shows form to edit project name
 * Delete mode: Shows confirmation dialog
 */
import { ref, computed, watch } from 'vue'
import { useUiStore, useProjectsStore } from '@/stores'
import { useCreateProject, useUpdateProject, useDeleteProject } from '@/composables/useProjects'
import Dialog from '@/components/ui/Dialog.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Loading from '@/components/ui/Loading.vue'
import RepoPickerDialog from './RepoPickerDialog.vue'
import type { Project, Repo } from '@/types'

const uiStore = useUiStore()
const projectsStore = useProjectsStore()

const createMutation = useCreateProject()
const updateMutation = useUpdateProject()
const deleteMutation = useDeleteProject()

// Form state
const formData = ref({
  name: '',
  repositories: [] as { display_name: string; git_repo_path: string }[],
})

// Repo picker dialog state
const showRepoPicker = ref(false)
const isCreatingProject = ref(false)

// Computed
const isCreateMode = computed(() => uiStore.activeDialog === 'createProject')
const isEditMode = computed(() => uiStore.activeDialog === 'editProject')
const isDeleteMode = computed(() => uiStore.activeDialog === 'deleteProject')

// For create mode, show repo picker first
const isFormOpen = computed(() => {
  if (isCreateMode.value) {
    return isCreatingProject.value // Only show form after repo is selected
  }
  return isEditMode.value || isDeleteMode.value
})

// Show repo picker when in create mode
watch(
  () => uiStore.activeDialog,
  (dialog) => {
    if (dialog === 'createProject') {
      showRepoPicker.value = true
      isCreatingProject.value = false
      formData.value = { name: '', repositories: [] }
    }
  },
  { immediate: true }
)

const dialogTitle = computed(() => {
  if (isCreateMode.value) return '创建项目'
  if (isEditMode.value) return '编辑项目'
  if (isDeleteMode.value) return '删除项目'
  return ''
})

const editingProject = computed(() => {
  if ((isEditMode.value || isDeleteMode.value) && uiStore.dialogData) {
    return uiStore.dialogData as Project
  }
  return null
})

const isSubmitting = computed(() =>
  createMutation.isPending.value ||
  updateMutation.isPending.value ||
  deleteMutation.isPending.value
)

const canSubmit = computed(() => {
  if (isDeleteMode.value) return true
  return formData.value.name.trim().length > 0
})

// Watch for dialog data changes to populate form
watch(
  () => [uiStore.activeDialog, uiStore.dialogData],
  () => {
    if (isEditMode.value && editingProject.value) {
      formData.value = {
        name: editingProject.value.name,
        repositories: [],
      }
    }
  },
  { immediate: true }
)

// Handle repo selection from RepoPickerDialog
const handleRepoSelected = async (repo: Repo) => {
  showRepoPicker.value = false

  // Use repo info to create project
  const projectName = repo.display_name || repo.name
  formData.value = {
    name: projectName,
    repositories: [{ display_name: projectName, git_repo_path: repo.path }],
  }

  // Auto-create project with selected repo
  try {
    const newProject = await createMutation.mutateAsync({
      name: formData.value.name,
      repositories: formData.value.repositories,
    })
    projectsStore.setCurrentProject(newProject.id)
    uiStore.closeDialog()
  } catch (error) {
    console.error('Project creation failed:', error)
    // Show form to allow manual retry
    isCreatingProject.value = true
  }
}

const handleRepoPickerCancel = () => {
  showRepoPicker.value = false
  uiStore.closeDialog()
}

// Methods
const handleSubmit = async () => {
  if (!canSubmit.value || isSubmitting.value) return

  try {
    if (isCreateMode.value) {
      const newProject = await createMutation.mutateAsync({
        name: formData.value.name.trim(),
        repositories: formData.value.repositories,
      })
      // Auto-select the new project
      projectsStore.setCurrentProject(newProject.id)
      uiStore.closeDialog()
    } else if (isEditMode.value && editingProject.value) {
      await updateMutation.mutateAsync({
        id: editingProject.value.id,
        data: { name: formData.value.name.trim() },
      })
      uiStore.closeDialog()
    } else if (isDeleteMode.value && editingProject.value) {
      await deleteMutation.mutateAsync(editingProject.value.id)
      // If deleted project was current, select another
      if (projectsStore.currentProjectId === editingProject.value.id) {
        projectsStore.setCurrentProject(null)
      }
      uiStore.closeDialog()
    }
  } catch (error) {
    console.error('Project operation failed:', error)
  }
}

const handleClose = () => {
  uiStore.closeDialog()
}
</script>

<template>
  <!-- Repo Picker Dialog (shown first in create mode) -->
  <RepoPickerDialog
    v-model:open="showRepoPicker"
    title="创建项目"
    description="选择或创建一个 Git 仓库作为项目基础"
    @select="handleRepoSelected"
    @cancel="handleRepoPickerCancel"
  />

  <!-- Main Form Dialog -->
  <Dialog :open="isFormOpen" :title="dialogTitle" @update:open="handleClose">
    <!-- Creating indicator (auto-create after repo selection) -->
    <div v-if="isCreateMode && createMutation.isPending.value && !isCreatingProject" class="flex flex-col items-center py-8">
      <Loading size="lg" />
      <p class="mt-4 text-text-muted">正在创建项目...</p>
    </div>

    <!-- Delete Confirmation -->
    <template v-else-if="isDeleteMode">
      <div class="space-y-4">
        <p class="text-text-secondary">
          确定要删除项目 <strong class="text-text-primary">{{ editingProject?.name }}</strong> 吗？
        </p>
        <p class="text-sm text-red-500">
          此操作不可撤销，项目下的所有任务和工作区都将被删除。
        </p>
      </div>
    </template>

    <!-- Edit Form / Manual Create Form -->
    <template v-else>
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <!-- Project Name -->
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">
            项目名称 <span class="text-red-500">*</span>
          </label>
          <Input
            v-model="formData.name"
            placeholder="输入项目名称"
            :disabled="isSubmitting"
            autofocus
          />
        </div>

        <!-- Show selected repo info in create mode -->
        <div v-if="isCreateMode && formData.repositories.length > 0">
          <label class="block text-sm font-medium text-text-secondary mb-1">
            关联仓库
          </label>
          <div class="p-3 bg-bg-secondary rounded-lg">
            <div class="text-sm font-medium text-text-primary">
              {{ formData.repositories[0].display_name }}
            </div>
            <div class="text-xs text-text-muted truncate mt-1">
              {{ formData.repositories[0].git_repo_path }}
            </div>
          </div>
        </div>

        <!-- Error message -->
        <div v-if="createMutation.isError.value || updateMutation.isError.value" class="p-3 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm rounded-lg">
          {{ createMutation.error.value?.message || updateMutation.error.value?.message || '操作失败' }}
        </div>
      </form>
    </template>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" :disabled="isSubmitting" @click="handleClose">
          取消
        </Button>
        <Button
          :variant="isDeleteMode ? 'danger' : 'primary'"
          :loading="isSubmitting"
          :disabled="!canSubmit || isSubmitting"
          @click="handleSubmit"
        >
          {{ isDeleteMode ? '删除' : isCreateMode ? '创建' : '保存' }}
        </Button>
      </div>
    </template>
  </Dialog>
</template>
