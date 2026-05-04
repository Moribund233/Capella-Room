import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import { useThemeStore, useNotificationStore } from '@/store'

const app = createApp(App)

app.use(createPinia())
app.use(router)

// 初始化主题（在挂载前执行，避免闪烁）
const themeStore = useThemeStore()
themeStore.initTheme()

// 初始化 WebSocket 通知监听器
const notificationStore = useNotificationStore()
notificationStore.initWebSocketListeners()

app.mount('#app')
