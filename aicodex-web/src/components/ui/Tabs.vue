<script setup lang="ts">
import { provide, computed } from 'vue'

interface Tab {
  name: string
  label: string
  disabled?: boolean
}

const props = defineProps<{
  tabs: Tab[]
  modelValue?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const activeTab = computed({
  get: () => props.modelValue || props.tabs[0]?.name,
  set: (value) => emit('update:modelValue', value)
})

const selectTab = (tab: Tab) => {
  if (!tab.disabled) {
    activeTab.value = tab.name
  }
}

provide('activeTab', activeTab)
</script>

<template>
  <div>
    <!-- Tab List -->
    <div class="border-b border-border-normal">
      <nav class="flex -mb-px space-x-4" role="tablist">
        <button
          v-for="tab in tabs"
          :key="tab.name"
          type="button"
          role="tab"
          :aria-selected="activeTab === tab.name"
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
          :class="{
            'border-brand text-brand': activeTab === tab.name,
            'border-transparent text-text-muted hover:text-text-primary hover:border-border-normal': activeTab !== tab.name,
            'opacity-50 cursor-not-allowed': tab.disabled
          }"
          :disabled="tab.disabled"
          @click="selectTab(tab)"
        >
          {{ tab.label }}
        </button>
      </nav>
    </div>

    <!-- Tab Panels -->
    <div class="mt-4">
      <slot :active-tab="activeTab" />
    </div>
  </div>
</template>
