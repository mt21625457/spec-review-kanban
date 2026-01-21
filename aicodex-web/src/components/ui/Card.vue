<script setup lang="ts">
defineProps<{
  variant?: 'default' | 'bordered' | 'elevated'
}>()
</script>

<template>
  <div
    class="card"
    :class="{
      'shadow-lg': variant === 'elevated',
      'border-2': variant === 'bordered'
    }"
  >
    <div v-if="$slots.header" class="card-header">
      <slot name="header" />
    </div>
    <div class="card-body">
      <slot />
    </div>
    <div v-if="$slots.footer" class="card-footer">
      <slot name="footer" />
    </div>
  </div>
</template>
