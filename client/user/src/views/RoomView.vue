<template>
  <div class="room-view">
    <!-- 移动端：在线用户抽屉按钮 -->
    <div class="mobile-header" v-if="isMobile">
      <n-button quaternary @click="showUsersDrawer = true">
        <template #icon>
          <Users :size="20" />
        </template>
        在线用户 ({{ onlineUsers.length }})
      </n-button>
    </div>

    <n-card :title="roomName" class="chat-card">
      <template #header-extra>
        <n-space :size="8" :wrap="false">
          <n-tag :type="wsStore.isConnected ? 'success' : 'error'" size="small">
            {{ wsStore.isConnected ? '已连接' : '未连接' }}
          </n-tag>
          <n-button size="small" @click="leaveRoom">
            离开
          </n-button>
        </n-space>
      </template>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="messages-container">
        <n-empty v-if="messages.length === 0" description="暂无消息，开始聊天吧！" />
        <div
          v-for="msg in messages"
          :key="msg.id"
          :class="['message-item', msg.type, { 'is-self': msg.sender?.id === currentUserId }]"
        >
          <div v-if="msg.type === 'system'" class="system-message">
            {{ msg.content }}
          </div>
          <template v-else>
            <div class="message-header">
              <n-avatar size="small">{{ msg.sender?.username?.charAt(0).toUpperCase() || '?' }}</n-avatar>
              <span class="sender-name">{{ msg.sender?.username || '未知用户' }}</span>
              <span class="message-time">{{ formatTime(msg.time) }}</span>
            </div>
            <div class="message-content">{{ msg.content }}</div>
          </template>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="input-area">
        <n-input-group>
          <n-input
            v-model:value="inputMessage"
            placeholder="输入消息..."
            :disabled="!wsStore.isConnected"
            @keyup.enter="sendMessage"
          />
          <n-button
            type="primary"
            :disabled="!wsStore.isConnected || !inputMessage.trim()"
            @click="sendMessage"
          >
            发送
          </n-button>
        </n-input-group>
      </div>
    </n-card>

    <!-- 桌面端：在线用户侧边栏 -->
    <n-card v-if="!isMobile" title="在线用户" class="users-card">
      <n-list>
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
          </n-thing>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- 移动端：在线用户抽屉 -->
    <n-drawer v-model:show="showUsersDrawer" :width="280" placement="right" v-if="isMobile">
      <n-drawer-content title="在线用户">
        <n-list>
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
            </n-thing>
          </n-list-item>
        </n-list>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Users } from 'lucide-vue-next'
import { NCard, NSpace, NTag, NButton, NEmpty, NAvatar, NList, NListItem, NThing, NBadge, NInputGroup, NInput, NDrawer, NDrawerContent } from 'naive-ui'
import { useAuthStore, useWebSocketStore } from '@/store'
import { getRoom } from '@/api/room'
import type { UserStatus } from '@/types/websocket'
import { useResponsive } from '@/composables/useResponsive'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const { isMobile } = useResponsive()

const roomId = computed(() => route.params.id as string)
const roomName = ref('聊天房间')
const inputMessage = ref('')
const messagesContainer = ref<HTMLDivElement | null>(null)
const showUsersDrawer = ref(false)

const messages = computed(() =>
  wsStore.chatMessages.filter(m => m.roomId === roomId.value)
)

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

function formatTime(time: string) {
  return new Date(time).toLocaleTimeString()
}

function sendMessage() {
  const content = inputMessage.value.trim()
  if (!content || !wsStore.isConnected) return

  wsStore.sendMessage(roomId.value, content)
  inputMessage.value = ''
}

function leaveRoom() {
  wsStore.leaveRoom(roomId.value)
  router.push('/rooms')
}

function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

// 监听消息变化，自动滚动
watch(() => wsStore.chatMessages.length, scrollToBottom)

onMounted(async () => {
  // 获取房间信息
  try {
    const room = await getRoom(roomId.value)
    roomName.value = room.name
  } catch (error) {
    console.error('获取房间信息失败:', error)
  }

  // 连接 WebSocket
  if (!wsStore.isConnected) {
    wsStore.connect()
  }

  // 加入房间
  wsStore.joinRoom(roomId.value)
})

onUnmounted(() => {
  // 离开房间
  wsStore.leaveRoom(roomId.value)
})
</script>

<style scoped>
.room-view {
  display: flex;
  gap: 24px;
  padding: 24px;
  height: calc(100vh - 64px);
}

.mobile-header {
  display: none;
}

.chat-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.users-card {
  width: 250px;
  flex-shrink: 0;
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  background: var(--bg-default);
  border-radius: 8px;
  margin-bottom: 16px;
  min-height: 400px;
  max-height: calc(100vh - 300px);
}

.message-item {
  margin-bottom: 16px;
}

.message-item.is-self {
  text-align: right;
}

.message-item.is-self .message-header {
  justify-content: flex-end;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.sender-name {
  font-weight: bold;
  font-size: 14px;
}

.message-time {
  font-size: 12px;
  color: var(--text-secondary);
}

.message-content {
  padding: 8px 12px;
  background: var(--bg-container);
  border-radius: 8px;
  display: inline-block;
  max-width: 70%;
  word-break: break-word;
}

.message-item.is-self .message-content {
  background: var(--primary-color);
  color: white;
}

.system-message {
  text-align: center;
  color: var(--text-secondary);
  font-size: 12px;
  padding: 8px;
}

.input-area {
  display: flex;
  gap: 8px;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .room-view {
    flex-direction: column;
    padding: 12px;
    gap: 12px;
    height: calc(100vh - 56px);
  }

  .mobile-header {
    display: flex;
    justify-content: flex-end;
    padding: 0 4px;
  }

  .users-card {
    display: none;
  }

  .chat-card :deep(.n-card-header) {
    padding: 12px 16px;
  }

  .chat-card :deep(.n-card__content) {
    padding: 12px 16px;
  }

  .messages-container {
    padding: 12px;
    min-height: unset;
    max-height: calc(100vh - 220px);
  }

  .message-content {
    max-width: 85%;
    font-size: 14px;
  }

  .sender-name {
    font-size: 13px;
  }

  .message-time {
    font-size: 11px;
  }
}

/* 小屏幕手机适配 */
@media (max-width: 480px) {
  .room-view {
    padding: 8px;
  }

  .messages-container {
    padding: 8px;
    max-height: calc(100vh - 200px);
  }

  .message-content {
    max-width: 90%;
    padding: 6px 10px;
    font-size: 14px;
  }

  .message-header {
    gap: 6px;
  }
}
</style>
