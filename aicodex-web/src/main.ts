import { createApp } from 'vue'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import pinia from './stores'
import i18n from './i18n'
import clickOutsideDirective from './directives/clickOutside'
import { queryClient } from './lib/queryClient'

// 导入样式
import './styles/base.css'

const app = createApp(App)

app.use(pinia)
app.use(router)
app.use(i18n)
app.use(clickOutsideDirective)
app.use(VueQueryPlugin, { queryClient })

app.mount('#app')
