<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { NConfigProvider, NMessageProvider, NNotificationProvider, NDialogProvider, zhCN, dateZhCN, darkTheme } from 'naive-ui'
import { storeToRefs } from 'pinia'
import { themeOverrides, darkThemeOverrides } from '@/styles/native-ui-overrides'
import { AuthLayout, MainLayout } from '@/layouts'
import { useAuthStore } from '@/stores/auth'
import { useWebSocketStore } from '@/stores/websocket'
import { useNotificationStore } from '@/stores/notification'
import { useResponsive } from '@/composables/useResponsive'
import { useThemeStore } from '@/stores/theme'

const route = useRoute()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const notificationStore = useNotificationStore()
const themeStore = useThemeStore()
const { isMobile } = useResponsive()
const { isAuthenticated } = storeToRefs(authStore)

// Naive UI 主题配置
const naiveTheme = computed(() => themeStore.isDark ? darkTheme : null)
const currentThemeOverrides = computed(() => themeStore.isDark ? darkThemeOverrides : themeOverrides)

const layout = computed(() => {
  if (route.name === 'login' || route.name === 'register') {
    return AuthLayout
  }
  return MainLayout
})

// 全局 WebSocket 连接管理
function ensureGlobalWebSocketConnection() {
  if (isAuthenticated.value && !wsStore.isConnected && !wsStore.isConnecting) {
    console.log('[App] Ensuring global WebSocket connection...')
    wsStore.connect()
  }
}

// 监听认证状态变化，自动连接/断开 WebSocket
watch(isAuthenticated, (authenticated) => {
  if (authenticated) {
    ensureGlobalWebSocketConnection()
  } else {
    wsStore.disconnect()
  }
})

onMounted(() => {
  // 应用启动时，如果已登录则连接 WebSocket
  ensureGlobalWebSocketConnection()
  // 初始化全局通知系统
  notificationStore.initialize()
})

onUnmounted(() => {
  // 应用卸载时清理通知系统
  notificationStore.cleanup()
})

// 根据设备类型设置最大通知数量
const maxNotifications = computed(() => isMobile.value ? 1 : 2)
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="currentThemeOverrides" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <NNotificationProvider :max="maxNotifications" placement="top">
        <NDialogProvider>
          <component :is="layout">
            <router-view />
          </component>
        </NDialogProvider>
      </NNotificationProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>
