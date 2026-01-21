<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog, Button, Badge, Loading } from '@/components/ui'
import { useUserInstances, useAssignInstances } from '@/composables/useUsers'
import type { UserInfo, InstanceInfo } from '@/types'

const props = defineProps<{
  open: boolean
  user: UserInfo
  instances: InstanceInfo[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const userId = ref(props.user.id)

watch(() => props.user, (newUser) => {
  userId.value = newUser.id
})

const { data: userInstances, isLoading } = useUserInstances(userId)
const assignMutation = useAssignInstances()

const selectedInstanceIds = ref<Set<string>>(new Set())
const errorMessage = ref('')

// 当用户实例加载完成后，初始化选中状态
watch([() => props.open, userInstances], ([isOpen, instances]) => {
  if (isOpen && instances) {
    selectedInstanceIds.value = new Set(instances.map(i => i.id))
    errorMessage.value = ''
  }
})

const isInstanceSelected = (instanceId: string) => {
  return selectedInstanceIds.value.has(instanceId)
}

const toggleInstance = (instanceId: string) => {
  const newSet = new Set(selectedInstanceIds.value)
  if (newSet.has(instanceId)) {
    newSet.delete(instanceId)
  } else {
    newSet.add(instanceId)
  }
  selectedInstanceIds.value = newSet
}

const hasChanges = computed(() => {
  if (!userInstances.value) return false
  const currentIds = new Set(userInstances.value.map(i => i.id))
  if (currentIds.size !== selectedInstanceIds.value.size) return true
  for (const id of selectedInstanceIds.value) {
    if (!currentIds.has(id)) return true
  }
  return false
})

type BadgeVariant = 'default' | 'pending' | 'running' | 'completed' | 'failed' | 'cancelled' | 'success' | 'secondary' | 'brand' | 'danger' | 'warning'

const getStatusBadge = (status: InstanceInfo['status']): BadgeVariant => {
  const map: Record<string, BadgeVariant> = {
    running: 'success',
    stopped: 'secondary',
    starting: 'warning',
    stopping: 'warning',
    error: 'danger',
  }
  return map[status] || 'secondary'
}

const handleSubmit = async () => {
  errorMessage.value = ''
  try {
    await assignMutation.mutateAsync({
      userId: props.user.id,
      instanceIds: Array.from(selectedInstanceIds.value),
    })
    emit('update:open', false)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : '分配实例失败'
  }
}

const handleClose = () => {
  emit('update:open', false)
}
</script>

<template>
  <Dialog :open="open" @close="handleClose">
    <template #title>
      {{ t('admin.users.assignDialog.title', { name: user.display_name || user.username }) }}
    </template>

    <div class="space-y-4">
      <!-- 错误提示 -->
      <div
        v-if="errorMessage"
        class="p-3 bg-error/10 border border-error/20 rounded-lg text-error text-sm"
      >
        {{ errorMessage }}
      </div>

      <!-- 加载状态 -->
      <div v-if="isLoading" class="py-8">
        <Loading />
      </div>

      <!-- 实例列表 -->
      <div v-else-if="instances.length === 0" class="py-8 text-center text-text-muted">
        {{ t('admin.users.assignDialog.noInstances') }}
      </div>

      <div v-else class="space-y-2 max-h-96 overflow-y-auto">
        <label
          v-for="instance in instances"
          :key="instance.id"
          class="flex items-center gap-3 p-3 rounded-lg border border-border hover:bg-surface-hover cursor-pointer transition-colors"
          :class="{ 'bg-brand/5 border-brand/30': isInstanceSelected(instance.id) }"
        >
          <input
            type="checkbox"
            :checked="isInstanceSelected(instance.id)"
            @change="toggleInstance(instance.id)"
            class="w-4 h-4 text-brand focus:ring-brand rounded"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-text-primary">{{ instance.name }}</span>
              <Badge :variant="getStatusBadge(instance.status)" size="sm">
                {{ instance.status }}
              </Badge>
            </div>
            <div v-if="instance.description" class="text-sm text-text-muted truncate">
              {{ instance.description }}
            </div>
            <div class="text-xs text-text-muted mt-0.5">
              端口: {{ instance.port }}
              <span v-if="instance.user_count !== undefined">
                · {{ instance.user_count }} 个用户
              </span>
            </div>
          </div>
        </label>
      </div>

      <!-- 操作按钮 -->
      <div class="flex justify-between items-center pt-4 border-t border-border">
        <div class="text-sm text-text-muted">
          已选择 {{ selectedInstanceIds.size }} 个实例
        </div>
        <div class="flex gap-3">
          <Button variant="ghost" @click="handleClose">
            {{ t('common.cancel') }}
          </Button>
          <Button
            variant="primary"
            :loading="assignMutation.isPending.value"
            :disabled="!hasChanges"
            @click="handleSubmit"
          >
            {{ t('common.save') }}
          </Button>
        </div>
      </div>
    </div>
  </Dialog>
</template>
