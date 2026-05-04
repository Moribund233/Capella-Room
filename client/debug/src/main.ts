import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import { useThemeStore } from '@/store'

const app = createApp(App)

app.use(createPinia())
app.use(router)

// 初始化主题（在挂载前执行，避免闪烁）
const themeStore = useThemeStore()
themeStore.initTheme()

app.mount('#app')
