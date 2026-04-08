import { createApp } from 'vue'
import { createPinia } from 'pinia'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router from './router'

/**
 * 引入样式文件
 * 1. 主题配色（需在head中动态加载以支持切换）
 * 2. 基础样式
 * 3. 响应式样式
 * 4. 组件样式
 */
import './style/base.css'
import './style/breakpoints.css'
import './style/components.css'

const app = createApp(App)

// 全局注册所有 Element Plus 图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(createPinia())
app.use(router)

app.mount('#app')
