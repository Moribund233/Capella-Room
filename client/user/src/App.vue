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
@import './styles/responsive/breakpoints.css';
</style>
