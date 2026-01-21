<script setup lang="ts">
import { ref } from 'vue'

interface DropdownItem {
  key: string
  label: string
  icon?: string
  disabled?: boolean
  danger?: boolean
  divider?: boolean
}

defineProps<{
  items: DropdownItem[]
  align?: 'left' | 'right'
}>()

const emit = defineEmits<{
  select: [key: string]
}>()

const isOpen = ref(false)

const toggleDropdown = () => {
  isOpen.value = !isOpen.value
}

const closeDropdown = () => {
  isOpen.value = false
}

const selectItem = (item: DropdownItem) => {
  if (!item.disabled && !item.divider) {
    emit('select', item.key)
    closeDropdown()
  }
}
</script>

<template>
  <div class="relative" v-click-outside="closeDropdown">
    <div @click="toggleDropdown">
      <slot name="trigger" :is-open="isOpen" />
    </div>

    <Transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <div
        v-if="isOpen"
        class="absolute z-50 mt-2 w-48 bg-bg-primary border border-border-normal rounded-lg shadow-lg py-1"
        :class="{
          'left-0': align === 'left' || !align,
          'right-0': align === 'right'
        }"
      >
        <template v-for="item in items" :key="item.key">
          <div
            v-if="item.divider"
            class="my-1 border-t border-border-normal"
          />
          <button
            v-else
            type="button"
            class="w-full px-4 py-2 text-left text-sm flex items-center gap-2 transition-colors"
            :class="{
              'text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20': item.danger,
              'text-text-primary hover:bg-bg-hover': !item.danger,
              'opacity-50 cursor-not-allowed': item.disabled
            }"
            :disabled="item.disabled"
            @click="selectItem(item)"
          >
            <span v-if="item.icon" class="w-4 h-4" v-html="item.icon" />
            {{ item.label }}
          </button>
        </template>
      </div>
    </Transition>
  </div>
</template>
