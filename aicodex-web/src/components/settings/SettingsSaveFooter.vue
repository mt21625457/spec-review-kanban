<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui'
import { Check } from 'lucide-vue-next'

defineProps<{
  hasChanges: boolean
  saving: boolean
  success: boolean
  error?: string | null
}>()

const emit = defineEmits<{
  save: []
  discard: []
}>()

const { t } = useI18n()
</script>

<template>
  <div
    v-if="hasChanges || success || error"
    class="sticky bottom-0 z-10 -mx-4 px-4 py-3 bg-bg-primary/80 backdrop-blur-sm border-t border-border-normal"
  >
    <div class="flex items-center justify-between gap-4">
      <!-- 左侧状态提示 -->
      <div class="flex items-center gap-2 text-sm">
        <template v-if="success">
          <Check class="w-4 h-4 text-green-500" />
          <span class="text-green-600 dark:text-green-400">
            {{ t('settings.save.success') }}
          </span>
        </template>
        <template v-else-if="error">
          <span class="text-red-600 dark:text-red-400">
            {{ error }}
          </span>
        </template>
        <template v-else-if="hasChanges">
          <span class="w-1.5 h-1.5 rounded-full bg-amber-500" />
          <span class="text-text-normal">
            {{ t('settings.save.unsavedChanges') }}
          </span>
        </template>
      </div>

      <!-- 右侧操作按钮 -->
      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          :disabled="!hasChanges || saving"
          @click="emit('discard')"
        >
          {{ t('settings.save.discard') }}
        </Button>
        <Button
          variant="primary"
          size="sm"
          :loading="saving"
          :disabled="!hasChanges || saving"
          @click="emit('save')"
        >
          {{ t('settings.save.button') }}
        </Button>
      </div>
    </div>
  </div>
</template>
