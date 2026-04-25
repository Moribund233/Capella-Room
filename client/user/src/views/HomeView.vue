<template>
  <div class="home-view">
    <n-card title="欢迎使用聊天室">
      <n-space vertical>
        <n-text>
          你好，<n-text strong>{{ authStore.username }}</n-text>！
        </n-text>
        <n-text>
          这是一个基于 WebSocket 的实时聊天应用。
        </n-text>
        <n-space>
          <n-button type="primary" @click="goToRooms">
            进入房间列表
          </n-button>
          <n-button @click="goToProfile">
            个人中心
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card title="连接状态" class="status-card">
      <n-space align="center">
        <n-tag :type="connectionStatusType">
          {{ connectionStatusText }}
        </n-tag>
        <n-button
          v-if="!wsStore.isConnected"
          size="small"
          @click="connectWebSocket"
        >
          重新连接
        </n-button>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NSpace, NText, NButton, NTag } from 'naive-ui'
import { Users, MessageSquare, Zap, Clock, Wifi } from 'lucide-vue-next'
import { useAuthStore, useWebSocketStore } from '@/store'
import { useStatusBar, useSystemStatus } from '@/composables'
import type { StatusItem } from '@/composables'

const router = useRouter()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const statusBar = useStatusBar()

// 使用真实系统状态数据
const { status: systemStatus } = useSystemStatus({
  interval: 5000,
  autoStart: true,
})

const connectionStatusType = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return 'success'
    case 'connecting':
    case 'reconnecting':
      return 'warning'
    default:
      return 'error'
  }
})

const connectionStatusText = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return '已连接'
    case 'connecting':
      return '连接中...'
    case 'reconnecting':
      return '重连中...'
    default:
      return '未连接'
  }
})

/** 当前时间 */
const currentTime = computed(() => {
  const now = new Date()
  return now.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
})

/** 状态栏项目 - 使用真实数据 */
const statusBarItems = computed<StatusItem[]>(() => [
  {
    icon: Users,
    label: '在线用户',
    value: systemStatus.value.onlineUsers,
    valueClass: 'text-primary',
  },
  {
    icon: MessageSquare,
    label: '活跃房间',
    value: systemStatus.value.activeRooms || '-',
    valueClass: 'text-success',
  },
  {
    icon: Zap,
    label: '延迟',
    value: `${systemStatus.value.latency}ms`,
    valueClass: systemStatus.value.latency < 50 ? 'text-success' : systemStatus.value.latency < 100 ? 'text-warning' : 'text-error',
  },
  {
    icon: Wifi,
    label: '连接状态',
    value: connectionStatusText.value,
    valueClass: wsStore.isConnected ? 'text-success' : 'text-error',
  },
  {
    icon: Clock,
    label: '当前时间',
    value: currentTime.value,
  },
])

// 监听状态变化，更新状态栏
watch(() => statusBarItems.value, (items) => {
  statusBar.setItems(items)
}, { immediate: true, deep: true })

function goToRooms() {
  router.push('/rooms')
}

function goToProfile() {
  router.push('/profile')
}

function connectWebSocket() {
  wsStore.connect()
}

onMounted(() => {
  // 自动连接 WebSocket
  if (!wsStore.isConnected) {
    wsStore.connect()
  }
})

onUnmounted(() => {
  // 清除状态栏内容
  statusBar.clear()
})
</script>

<style scoped>
.home-view {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
}

.status-card {
  margin-top: 24px;
}

/* 状态栏值的颜色类 */
:global(.text-primary) {
  color: var(--color-primary);
}

:global(.text-success) {
  color: var(--color-success);
}

:global(.text-warning) {
  color: var(--color-warning);
}

:global(.text-error) {
  color: var(--color-error);
}
</style>
