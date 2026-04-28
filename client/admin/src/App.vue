<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { NConfigProvider, NMessageProvider, NDialogProvider, NNotificationProvider } from 'naive-ui'
import { useTheme } from '@/composables'
import { initConfig } from '@/config'

/**
 * 主题配置
 * naiveTheme: Naive UI 内置主题（darkTheme 或 lightTheme）
 */
const { naiveTheme, initTheme } = useTheme()

/**
 * 预加载配置
 * 确保在应用启动时加载配置，避免各组件独立加载导致的时序问题
 */
initConfig()

/**
 * 初始化主题
 */
onMounted(() => {
  initTheme()
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
