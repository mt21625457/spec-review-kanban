<script setup lang="ts">
import { ref, computed } from 'vue'

interface Option {
  value: string | number
  label: string
  disabled?: boolean
}

const props = defineProps<{
  modelValue?: string | number | null
  options: Option[]
  placeholder?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const isOpen = ref(false)

const selectedOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue)
})

const toggleDropdown = () => {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
  }
}

const selectOption = (option: Option) => {
  if (!option.disabled) {
    emit('update:modelValue', option.value)
    isOpen.value = false
  }
}

const closeDropdown = () => {
  isOpen.value = false
}
</script>

<template>
  <div class="relative" v-click-outside="closeDropdown">
    <button
      type="button"
      class="input flex items-center justify-between w-full"
      :class="{ 'opacity-50 cursor-not-allowed': disabled }"
      :disabled="disabled"
      @click="toggleDropdown"
    >
      <span :class="{ 'text-text-muted': !selectedOption }">
        {{ selectedOption?.label || placeholder || '请选择' }}
      </span>
      <svg
        class="h-4 w-4 text-text-muted transition-transform"
        :class="{ 'rotate-180': isOpen }"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

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
        class="absolute z-50 mt-1 w-full bg-bg-primary border border-border-normal rounded-lg shadow-lg py-1 max-h-60 overflow-auto"
      >
        <button
          v-for="option in options"
          :key="option.value"
          type="button"
          class="w-full px-3 py-2 text-left text-sm hover:bg-bg-hover transition-colors"
          :class="{
            'bg-brand/10 text-brand': option.value === modelValue,
            'opacity-50 cursor-not-allowed': option.disabled
          }"
          :disabled="option.disabled"
          @click="selectOption(option)"
        >
          {{ option.label }}
        </button>
      </div>
    </Transition>
  </div>
</template>
