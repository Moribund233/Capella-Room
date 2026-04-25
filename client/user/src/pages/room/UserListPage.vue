<template>
  <div class="user-list-page">
    <n-card :title="`${roomName} - 在线用户`" class="users-card">
      <template #header-extra>
        <n-space :size="8" :wrap="false">
          <n-tag :type="wsStore.isConnected ? 'success' : 'error'" size="small">
            {{ wsStore.isConnected ? '已连接' : '未连接' }}
          </n-tag>
          <n-button size="small" @click="goBack">
            返回
          </n-button>
        </n-space>
      </template>

      <!-- 错误提示 -->
      <n-alert v-if="wsStore.lastError" type="error" :show-icon="false" style="margin-bottom: 16px;">
        {{ wsStore.lastError }}
      </n-alert>

      <!-- 在线用户列表 -->
      <n-spin :show="!wsStore.isConnected && !wsStore.lastError">
        <n-empty v-if="onlineUsers.length === 0" description="暂无在线用户" />
        <n-list v-else>
          <n-list-item v-for="user in onlineUsers" :key="user.id">
            <n-thing>
              <template #avatar>
                <n-avatar size="small">{{ user.username.charAt(0).toUpperCase() }}</n-avatar>
              </template>
              <template #header>
                {{ user.username }}
                <n-tag v-if="user.id === currentUserId" size="small" type="info">我</n-tag>
              </template>
              <template #header-extra>
                <n-badge :type="getStatusType(user.status)" />
              </template>
              <template #description>
                <n-text depth="3">{{ getStatusText(user.status) }}</n-text>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>
      </n-spin>

      <!-- 统计信息 -->
      <template #footer>
        <n-space justify="space-between">
          <n-text depth="3">在线人数: {{ onlineUsers.length }}</n-text>
          <n-text depth="3">房间ID: {{ roomId }}</n-text>
        </n-space>
      </template>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NCard,
  NSpace,
  NTag,
  NButton,
  NEmpty,
  NAvatar,
  NList,
  NListItem,
  NThing,
  NBadge,
  NText,
  NAlert,
  NSpin,
} from 'naive-ui'
import { useAuthStore, useWebSocketStore } from '@/store'
import { getRoom } from '@/api/room'
import type { UserStatus } from '@/types/websocket'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()

const roomId = computed(() => {
  const id = route.params.id
  return id && typeof id === 'string' && id !== 'undefined' ? id : ''
})
const roomName = ref('聊天房间')

const onlineUsers = computed(() => wsStore.onlineUsers)
const currentUserId = computed(() => authStore.user?.id)

function getStatusType(status: UserStatus) {
  switch (status) {
    case 'online':
      return 'success'
    case 'away':
      return 'warning'
    case 'busy':
      return 'error'
    default:
      return 'default'
  }
}

function getStatusText(status: UserStatus) {
  switch (status) {
    case 'online':
      return '在线'
    case 'away':
      return '离开'
    case 'busy':
      return '忙碌'
    case 'offline':
      return '离线'
    default:
      return '未知'
  }
}

function goBack() {
  router.push(`/room/chat/${roomId.value}`)
}

// 监听 WebSocket 连接状态，连接成功后自动加入房间
const unwatchConnected = watchEffect(() => {
  if (wsStore.isConnected && roomId.value) {
    wsStore.joinRoom(roomId.value)
  }
})

onMounted(async () => {
  // 获取房间信息
  try {
    const room = await getRoom(roomId.value)
    roomName.value = room.name
  } catch (error) {
    console.error('获取房间信息失败:', error)
  }

  // 连接 WebSocket（如果未连接）
  if (!wsStore.isConnected) {
    wsStore.connect()
  }
})

onUnmounted(() => {
  // 停止监听
  unwatchConnected()
  // 离开房间
  wsStore.leaveRoom(roomId.value)
})
</script>

<style scoped>
.user-list-page {
  padding: 24px;
  max-width: 600px;
  margin: 0 auto;
  height: calc(100vh - 64px);
}

.users-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.users-card :deep(.n-card__content) {
  flex: 1;
  overflow-y: auto;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .user-list-page {
    padding: 12px;
    height: calc(100vh - 56px);
  }
}

/* 小屏幕手机适配 */
@media (max-width: 480px) {
  .user-list-page {
    padding: 8px;
  }
}
</style>
