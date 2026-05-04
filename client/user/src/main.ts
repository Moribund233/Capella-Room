import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

import App from './App.vue'
import router from './router'
import { useConfigStore } from './stores/config'
import './styles/index.css'

async function bootstrap() {
  const app = createApp(App)

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)

  app.use(pinia)
  app.use(router)

  // 在 app 挂载前预取服务端配置（不阻塞挂载）
  const configStore = useConfigStore()
  configStore.fetchConfig()

  app.mount('#app')
}

bootstrap()
