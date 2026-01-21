<script setup lang="ts">
defineProps<{
  variant?: 'default' | 'pending' | 'running' | 'completed' | 'failed' | 'cancelled' | 'success' | 'secondary' | 'brand' | 'danger' | 'warning'
  size?: 'sm' | 'md'
}>()
</script>

<template>
  <span
    class="badge"
    :class="[
      // Size
      size === 'sm' ? 'text-xs px-1.5 py-0.5' : 'text-xs px-2 py-1',
      // Variant
      {
        'badge-pending': variant === 'pending',
        'badge-running': variant === 'running',
        'badge-completed': variant === 'completed',
        'badge-failed': variant === 'failed',
        'badge-cancelled': variant === 'cancelled',
        'bg-success/20 text-success': variant === 'success',
        'bg-text-muted/20 text-text-muted': variant === 'secondary',
        'bg-brand/20 text-brand': variant === 'brand',
        'bg-error/20 text-error': variant === 'danger',
        'bg-warning/20 text-warning': variant === 'warning',
        'bg-bg-secondary text-text-normal': !variant || variant === 'default'
      }
    ]"
  >
    <slot />
  </span>
</template>
