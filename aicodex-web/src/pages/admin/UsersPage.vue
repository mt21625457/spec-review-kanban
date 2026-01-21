<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { PageHeader } from '@/components/layout'
import { Card, Button, Badge, Input, Loading } from '@/components/ui'
import { useUsers, useDeleteUser, useSetUserActive } from '@/composables/useUsers'
import { useAdminInstances } from '@/composables/useInstances'
import type { UserInfo } from '@/types'
import CreateUserDialog from '@/components/admin/CreateUserDialog.vue'
import AssignInstancesDialog from '@/components/admin/AssignInstancesDialog.vue'

const { t } = useI18n()

const { data: users, isLoading, refetch } = useUsers()
const { data: instances } = useAdminInstances()
const deleteUserMutation = useDeleteUser()
const setActiveMutation = useSetUserActive()

const searchQuery = ref('')
const showCreateDialog = ref(false)
const showAssignDialog = ref(false)
const selectedUser = ref<UserInfo | null>(null)

const filteredUsers = computed(() => {
  if (!users.value) return []
  if (!searchQuery.value) return users.value
  const query = searchQuery.value.toLowerCase()
  return users.value.filter(
    (u) =>
      u.username.toLowerCase().includes(query) ||
      u.email?.toLowerCase().includes(query) ||
      u.display_name?.toLowerCase().includes(query)
  )
})

const handleDeleteUser = async (user: UserInfo) => {
  if (!confirm(`确定要删除用户 "${user.username}" 吗？此操作不可撤销。`)) return
  try {
    await deleteUserMutation.mutateAsync(user.id)
  } catch (error) {
    console.error('删除用户失败:', error)
  }
}

const handleToggleActive = async (user: UserInfo) => {
  try {
    await setActiveMutation.mutateAsync({
      userId: user.id,
      isActive: !user.is_active,
    })
  } catch (error) {
    console.error('操作失败:', error)
  }
}

const openAssignDialog = (user: UserInfo) => {
  selectedUser.value = user
  showAssignDialog.value = true
}

const getRoleBadge = (role: string) => {
  return role === 'admin' ? 'brand' : 'secondary'
}

const formatDate = (dateStr: string | null) => {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleString('zh-CN')
}
</script>

<template>
  <div>
    <PageHeader :title="t('admin.users.title')">
      <template #actions>
        <Button variant="primary" @click="showCreateDialog = true">
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t('admin.users.create') }}
        </Button>
      </template>
    </PageHeader>

    <!-- 搜索和过滤 -->
    <div class="mb-6 flex items-center gap-4">
      <div class="flex-1 max-w-md">
        <Input
          v-model="searchQuery"
          :placeholder="t('admin.users.searchPlaceholder')"
          class="w-full"
        />
      </div>
      <Button variant="ghost" @click="refetch()">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </Button>
    </div>

    <!-- 用户列表 -->
    <div v-if="isLoading" class="py-12">
      <Loading />
    </div>

    <div v-else-if="!filteredUsers.length" class="py-12 text-center text-text-muted">
      {{ t('admin.users.empty') }}
    </div>

    <div v-else class="grid gap-4">
      <Card v-for="user in filteredUsers" :key="user.id" class="hover:shadow-md transition-shadow">
        <div class="flex items-center gap-4">
          <!-- 头像 -->
          <div
            class="w-12 h-12 rounded-full flex items-center justify-center text-lg font-medium"
            :class="user.is_active ? 'bg-brand/20 text-brand' : 'bg-text-muted/20 text-text-muted'"
          >
            {{ (user.display_name || user.username).charAt(0).toUpperCase() }}
          </div>

          <!-- 用户信息 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-text-primary">
                {{ user.display_name || user.username }}
              </span>
              <Badge :variant="getRoleBadge(user.role)">
                {{ user.role === 'admin' ? '管理员' : '用户' }}
              </Badge>
              <Badge v-if="!user.is_active" variant="danger">
                已停用
              </Badge>
            </div>
            <div class="text-sm text-text-muted mt-0.5">
              @{{ user.username }}
              <span v-if="user.email"> · {{ user.email }}</span>
            </div>
            <div class="text-xs text-text-muted mt-1">
              最后登录: {{ formatDate(user.last_login_at) }}
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              :title="t('admin.users.assignInstances')"
              @click="openAssignDialog(user)"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="user.is_active ? t('admin.users.deactivate') : t('admin.users.activate')"
              @click="handleToggleActive(user)"
            >
              <svg v-if="user.is_active" class="w-4 h-4 text-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
              </svg>
              <svg v-else class="w-4 h-4 text-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :title="t('common.delete')"
              @click="handleDeleteUser(user)"
            >
              <svg class="w-4 h-4 text-error" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </Button>
          </div>
        </div>
      </Card>
    </div>

    <!-- 创建用户对话框 -->
    <CreateUserDialog v-model:open="showCreateDialog" />

    <!-- 分配实例对话框 -->
    <AssignInstancesDialog
      v-if="selectedUser"
      v-model:open="showAssignDialog"
      :user="selectedUser"
      :instances="instances || []"
    />
  </div>
</template>
