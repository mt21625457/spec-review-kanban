<script setup lang="ts">
import { inject, computed, type Ref } from 'vue'

const props = defineProps<{
  name: string
}>()

const activeTab = inject<Ref<string>>('activeTab')

const isActive = computed(() => activeTab?.value === props.name)
</script>

<template>
  <div v-show="isActive" role="tabpanel">
    <slot />
  </div>
</template>
