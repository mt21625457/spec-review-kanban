import { computed } from 'vue'
import { useI18n as useVueI18n } from 'vue-i18n'

export function useLocale() {
  const { locale, t } = useVueI18n()

  const currentLocale = computed(() => locale.value)

  const setLocale = (newLocale: 'zh-CN' | 'en-US') => {
    locale.value = newLocale
    localStorage.setItem('locale', newLocale)
  }

  const availableLocales = [
    { code: 'zh-CN', name: '简体中文' },
    { code: 'en-US', name: 'English' },
  ]

  return {
    currentLocale,
    setLocale,
    availableLocales,
    t,
  }
}
