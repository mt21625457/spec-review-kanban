<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<template>
  <div class="mb-6">
    <h1 class="text-2xl font-bold text-text-high">{{ title }}</h1>
    <p v-if="subtitle" class="mt-1 text-sm text-text-low">{{ subtitle }}</p>
    <div v-if="$slots.actions" class="mt-4 flex items-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
