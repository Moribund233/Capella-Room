<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { NConfigProvider, NMessageProvider, NDialogProvider, NNotificationProvider } from 'naive-ui'
import { useTheme } from '@/composables'
import { useAuthStore } from '@/store'
import { initConfig } from '@/config'

/**
 * 主题配置
 * naiveTheme: Naive UI 内置主题（darkTheme 或 lightTheme）
 */
const { naiveTheme, initTheme } = useTheme()

/**
 * 认证 Store
 */
const authStore = useAuthStore()

/**
 * 预加载配置
 * 确保在应用启动时加载配置，避免各组件独立加载导致的时序问题
 */
initConfig()

/**
 * 初始化应用
 * - 初始化主题
 * - 初始化认证状态（包括 WebSocket 连接）
 */
onMounted(() => {
  initTheme()
  // 初始化认证状态，如果已登录则自动建立 WebSocket 连接
  authStore.initAuth()
})
</script>

<template>
  <n-config-provider :theme="naiveTheme">
    <n-message-provider>
      <n-notification-provider>
        <n-dialog-provider>
          <RouterView />
        </n-dialog-provider>
      </n-notification-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
/* 导入全局样式 */
@import './styles/global/color.css';
@import './styles/global/base.css';
@import './styles/global/animations.css';
@import './styles/responsive/breakpoints.css';

/* 全局字体优化 */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');

/* 全局背景氛围 */
.app-container::before {
  content: '';
  position: fixed;
  top: -30%;
  right: -10%;
  width: 70vw;
  height: 70vw;
  background: radial-gradient(circle,
      rgba(99, 102, 241, 0.04) 0%,
      transparent 60%);
  pointer-events: none;
  z-index: -1;
}

.app-container::after {
  content: '';
  position: fixed;
  bottom: -20%;
  left: -5%;
  width: 50vw;
  height: 50vw;
  background: radial-gradient(circle,
      rgba(139, 92, 246, 0.03) 0%,
      transparent 50%);
  pointer-events: none;
  z-index: -1;
}
</style>
