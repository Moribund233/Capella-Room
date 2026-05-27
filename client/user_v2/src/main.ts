import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

import App from './App.vue'
import router from './router'
import i18n from './i18n'
import { useConfigStore, useThemeStore, usePersonalizationStore } from './stores'
import { initErrorHandler } from './services/error'

// Element Plus 基础样式（必须在自定义变量之前）
import 'element-plus/dist/index.css'
// Element Plus 自定义变量
import '@/styles/element-variables.scss'

async function bootstrap() {
  const app = createApp(App)

  // 初始化全局错误处理
  initErrorHandler(app)

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)

  app.use(pinia)
  app.use(router)
  app.use(i18n)

  // 初始化主题（在挂载前执行，避免闪烁）
  const themeStore = useThemeStore()
  themeStore.initTheme()
  themeStore.watchSystemThemeChange()

  // 初始化个性化配置
  const personalizationStore = usePersonalizationStore()
  personalizationStore.initPersonalization()

  // 在 app 挂载前预取服务端配置（不阻塞挂载）
  const configStore = useConfigStore()
  configStore.fetchConfig()

  app.mount('#app')
}

bootstrap()
