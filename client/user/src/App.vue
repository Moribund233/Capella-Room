<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { NConfigProvider, NMessageProvider, NNotificationProvider, zhCN, dateZhCN } from 'naive-ui'
import { storeToRefs } from 'pinia'
import { themeOverrides } from '@/styles/native-ui-overrides'
import { AuthLayout, MainLayout } from '@/layouts'
import { useAuthStore } from '@/stores/auth'
import { useWebSocketStore } from '@/stores/websocket'
import { useNotificationStore } from '@/stores/notification'
import { useResponsive } from '@/composables/useResponsive'

const route = useRoute()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const notificationStore = useNotificationStore()
const { isMobile } = useResponsive()
const { isAuthenticated } = storeToRefs(authStore)

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
  <NConfigProvider :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <NNotificationProvider :max="maxNotifications" placement="top">
        <component :is="layout">
          <router-view />
        </component>
      </NNotificationProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>
