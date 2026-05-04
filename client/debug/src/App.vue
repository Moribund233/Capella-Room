<script setup lang="ts">
import { RouterView } from 'vue-router'
import { NConfigProvider, NMessageProvider, NDialogProvider, NNotificationProvider } from 'naive-ui'
import { useTheme } from '@/composables'
import { initConfig, initWebSocketConfig } from '@/config'

/**
 * 主题配置
 * naiveTheme: Naive UI 内置主题（darkTheme 或 lightTheme）
 */
const { naiveTheme } = useTheme()

/**
 * 预加载配置
 * 确保在应用启动时加载配置，避免各组件独立加载导致的时序问题
 */
initConfig()

/**
 * 初始化 WebSocket 配置
 * 从服务器获取 WebSocket 心跳等配置，确保前后端配置一致
 */
initWebSocketConfig().catch(err => {
  console.warn('[App] WebSocket 配置初始化失败，使用默认配置:', err)
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
