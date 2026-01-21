<script setup lang="ts">
import { ref } from 'vue'
import { useLocale } from '@/composables/useLocale'

const { currentLocale, setLocale, availableLocales } = useLocale()
const isOpen = ref(false)

const selectLocale = (code: 'zh-CN' | 'en-US') => {
  setLocale(code)
  isOpen.value = false
}
</script>

<template>
  <div class="relative">
    <button
      class="p-2 rounded-md text-text-normal hover:bg-bg-secondary transition-colors"
      @click="isOpen = !isOpen"
      title="切换语言"
    >
      <span class="text-lg">🌐</span>
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 mt-2 w-36 bg-bg-primary border border-border rounded-lg shadow-lg py-1 z-50"
    >
      <button
        v-for="locale in availableLocales"
        :key="locale.code"
        class="w-full px-4 py-2 text-left text-sm hover:bg-bg-secondary transition-colors"
        :class="currentLocale === locale.code ? 'text-brand' : 'text-text-normal'"
        @click="selectLocale(locale.code as 'zh-CN' | 'en-US')"
      >
        {{ locale.name }}
      </button>
    </div>
  </div>

  <!-- 点击外部关闭 -->
  <div
    v-if="isOpen"
    class="fixed inset-0 z-40"
    @click="isOpen = false"
  />
</template>
