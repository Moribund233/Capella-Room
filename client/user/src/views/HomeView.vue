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
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NSpace, NText, NButton, NTag } from 'naive-ui'
import { useAuthStore, useWebSocketStore } from '@/store'

const router = useRouter()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()

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
</style>
