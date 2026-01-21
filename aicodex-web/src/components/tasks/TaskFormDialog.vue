<script setup lang="ts">
/**
 * TaskFormDialog - Dialog for creating, editing, and deleting tasks
 */
import { ref, computed, watch } from 'vue'
import { useUiStore, useProjectsStore } from '@/stores'
import { useCreateTask, useUpdateTask, useDeleteTask, taskStatusInfo } from '@/composables/useTasks'
import Dialog from '@/components/ui/Dialog.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Select from '@/components/ui/Select.vue'
import type { Task, TaskStatus } from '@/types'

const uiStore = useUiStore()
const projectsStore = useProjectsStore()

const createMutation = useCreateTask()
const updateMutation = useUpdateTask()
const deleteMutation = useDeleteTask()

// Form state
const formData = ref({
  title: '',
  description: '',
  status: 'todo' as TaskStatus,
})

// Dialog state
const isCreateMode = computed(() => uiStore.activeDialog === 'createTask')
const isEditMode = computed(() => uiStore.activeDialog === 'editTask')
const isDeleteMode = computed(() => uiStore.activeDialog === 'deleteTask')
const isOpen = computed(() => isCreateMode.value || isEditMode.value || isDeleteMode.value)

const dialogTitle = computed(() => {
  if (isCreateMode.value) return '新建任务'
  if (isEditMode.value) return '编辑任务'
  if (isDeleteMode.value) return '删除任务'
  return ''
})

const editingTask = computed(() => {
  if ((isEditMode.value || isDeleteMode.value) && uiStore.dialogData) {
    return uiStore.dialogData as Task
  }
  return null
})

const initialStatus = computed(() => {
  if (isCreateMode.value && uiStore.dialogData && typeof uiStore.dialogData === 'object') {
    const data = uiStore.dialogData as { status?: TaskStatus }
    return data.status || 'todo'
  }
  return 'todo'
})

const isSubmitting = computed(() =>
  createMutation.isPending.value ||
  updateMutation.isPending.value ||
  deleteMutation.isPending.value
)

const canSubmit = computed(() => {
  if (isDeleteMode.value) return true
  return formData.value.title.trim().length > 0
})

// Status options for select
const statusOptions = computed(() =>
  Object.entries(taskStatusInfo).map(([value, info]) => ({
    value,
    label: info.label,
  }))
)

// Watch for dialog data changes
watch(
  () => [uiStore.activeDialog, uiStore.dialogData],
  () => {
    if (isEditMode.value && editingTask.value) {
      formData.value = {
        title: editingTask.value.title,
        description: editingTask.value.description || '',
        status: editingTask.value.status,
      }
    } else if (isCreateMode.value) {
      formData.value = {
        title: '',
        description: '',
        status: initialStatus.value,
      }
    }
  },
  { immediate: true }
)

// Methods
const handleSubmit = async () => {
  if (!canSubmit.value || isSubmitting.value) return

  try {
    if (isCreateMode.value) {
      if (!projectsStore.currentProjectId) {
        console.error('No project selected')
        return
      }
      await createMutation.mutateAsync({
        project_id: projectsStore.currentProjectId,
        title: formData.value.title.trim(),
        description: formData.value.description.trim() || null,
        status: formData.value.status,
      })
      uiStore.closeDialog()
    } else if (isEditMode.value && editingTask.value) {
      await updateMutation.mutateAsync({
        taskId: editingTask.value.id,
        data: {
          title: formData.value.title.trim(),
          description: formData.value.description.trim() || null,
          status: formData.value.status,
        },
      })
      uiStore.closeDialog()
    } else if (isDeleteMode.value && editingTask.value) {
      await deleteMutation.mutateAsync(editingTask.value.id)
      // Close task panel if it was showing the deleted task
      if (uiStore.selectedTaskId === editingTask.value.id) {
        uiStore.closePanel()
      }
      uiStore.closeDialog()
    }
  } catch (error) {
    console.error('Task operation failed:', error)
  }
}

const handleClose = () => {
  uiStore.closeDialog()
}
</script>

<template>
  <Dialog :open="isOpen" :title="dialogTitle" size="md" @update:open="handleClose">
    <!-- Delete Confirmation -->
    <template v-if="isDeleteMode">
      <div class="space-y-4">
        <p class="text-text-secondary">
          确定要删除任务 <strong class="text-text-primary">{{ editingTask?.title }}</strong> 吗？
        </p>
        <p class="text-sm text-red-500">
          此操作不可撤销，任务的所有工作区和执行记录都将被删除。
        </p>
      </div>
    </template>

    <!-- Create/Edit Form -->
    <template v-else>
      <form @submit.prevent="handleSubmit" class="space-y-4">
        <!-- Task Title -->
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">
            任务标题 <span class="text-red-500">*</span>
          </label>
          <Input
            v-model="formData.title"
            placeholder="输入任务标题"
            :disabled="isSubmitting"
            autofocus
          />
        </div>

        <!-- Task Description -->
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">
            任务描述
          </label>
          <textarea
            v-model="formData.description"
            class="input min-h-[100px] resize-y"
            placeholder="输入任务描述（可选）"
            :disabled="isSubmitting"
          />
        </div>

        <!-- Task Status -->
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-1">
            状态
          </label>
          <Select
            v-model="formData.status"
            :options="statusOptions"
            :disabled="isSubmitting"
          />
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
