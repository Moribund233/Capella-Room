import { createApp } from 'vue'
import { createPinia } from 'pinia'
import naive from 'naive-ui'

import App from './App.vue'
import router from './router'
import { useMultiUserAuthStore } from '@/stores/multiUserAuth'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(naive)

// 初始化多用户认证状态（从 sessionStorage 恢复并重建 WebSocket 连接）
const multiUserAuthStore = useMultiUserAuthStore()
multiUserAuthStore.initialize().then(() => {
  app.mount('#app')
}).catch((error) => {
  console.error('[Main] 初始化多用户认证状态失败:', error)
  app.mount('#app')
})
