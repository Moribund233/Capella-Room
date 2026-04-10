<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { wsClient } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { WebSocketMessage, ConnectionStatus } from '@/types/websocket'
import {
  Wifi,
  WifiOff,
  Send,
  Play,
  Square,
  Trash2,
  MessageSquare,
  Users,
  RefreshCw,
  LogIn,
  LogOut,
  MessageCircle,
} from 'lucide-vue-next'

const authStore = useAuthStore()

// ========== 连接状态 ==========
const connectionStatus = ref<ConnectionStatus>('disconnected')
const latency = ref<number | null>(null)
const lastPingTime = ref<number | null>(null)
const joinedRooms = ref<string[]>([])
const onlineUsers = ref<number>(0)

// ========== 房间管理 ==========
const roomIdInput = ref('')
const currentRoom = ref<string | null>(null)

// ========== 消息 ==========
const chatInput = ref('')
const replyToMessageId = ref<string | null>(null)
const messages = ref<
  Array<{
    id: string
    type: 'system' | 'sent' | 'received'
    content: string
    time: string
    sender?: string
    roomId?: string
  }>
>([
  {
    id: '0',
    type: 'system',
    content: 'WebSocket 测试页面已加载，点击"连接"开始测试',
    time: new Date().toLocaleTimeString(),
  },
])

// ========== 计算属性 ==========
const isConnected = computed(() => connectionStatus.value === 'connected')
const isConnecting = computed(() => connectionStatus.value === 'connecting')
const isReconnecting = computed(() => connectionStatus.value === 'reconnecting')

const statusText = computed(() => {
  switch (connectionStatus.value) {
    case 'connected':
      return '已连接'
    case 'connecting':
      return '连接中...'
    case 'reconnecting':
      return '重连中...'
    case 'error':
      return '连接错误'
    default:
      return '未连接'
  }
})

const statusType = computed(() => {
  switch (connectionStatus.value) {
    case 'connected':
      return 'success'
    case 'connecting':
    case 'reconnecting':
      return 'warning'
    case 'error':
      return 'error'
    default:
      return 'default'
  }
})

// ========== WebSocket 事件处理 ==========
const handleConnect = () => {
  connectionStatus.value = 'connected'
  addSystemMessage('WebSocket 连接已建立')
  // 连接成功后获取在线用户
  wsClient.getOnlineUsers()
}

const handleDisconnect = () => {
  connectionStatus.value = 'disconnected'
  joinedRooms.value = []
  addSystemMessage('WebSocket 连接已断开')
}

const handleError = (error: Error) => {
  addSystemMessage(`连接错误: ${error.message}`, 'error')
}

const handleMessage = (message: WebSocketMessage) => {
  switch (message.type) {
    case 'AuthResult':
      if (message.payload.success) {
        addSystemMessage('认证成功')
      } else {
        addSystemMessage(`认证失败: ${message.payload.message}`, 'error')
      }
      break

    case 'Pong':
      if (lastPingTime.value) {
        latency.value = Date.now() - lastPingTime.value
        lastPingTime.value = null
      }
      break

    case 'RoomJoined':
      joinedRooms.value.push(message.payload.room_id)
      addSystemMessage(`已加入房间: ${message.payload.room_id}`)
      break

    case 'RoomLeft':
      joinedRooms.value = joinedRooms.value.filter((id) => id !== message.payload.room_id)
      addSystemMessage(`已离开房间: ${message.payload.room_id}`)
      break

    case 'UserJoined':
      addSystemMessage(`用户 ${message.payload.username} 加入房间`, 'info', message.payload.room_id)
      break

    case 'UserLeft':
      addSystemMessage(`用户 ${message.payload.username} 离开房间`, 'info', message.payload.room_id)
      break

    case 'NewMessage':
      messages.value.push({
        id: message.payload.message_id,
        type: 'received',
        content: message.payload.content,
        time: new Date(message.payload.created_at).toLocaleTimeString(),
        sender: message.payload.sender_name,
        roomId: message.payload.room_id,
      })
      break

    case 'GlobalOnlineUsers':
      onlineUsers.value = message.payload.total
      break

    case 'OnlineUsers':
      addSystemMessage(
        `房间 ${message.payload.room_id} 在线用户: ${message.payload.users.map((u) => u.username).join(', ')}`
      )
      break

    case 'Error':
      addSystemMessage(`错误: ${message.payload.message}`, 'error')
      break

    case 'SystemMessage':
      addSystemMessage(message.payload.content)
      break

    default:
      // 其他消息类型记录到控制台
      console.log('收到消息:', message)
  }
}

// ========== 消息辅助函数 ==========
const addSystemMessage = (content: string, level: 'info' | 'error' = 'info', roomId?: string) => {
  messages.value.push({
    id: Date.now().toString(),
    type: 'system',
    content,
    time: new Date().toLocaleTimeString(),
    roomId,
  })
}

// ========== 连接控制 ==========
const connectWebSocket = async () => {
  try {
    connectionStatus.value = 'connecting'
    wsClient.setHandlers({
      onConnect: handleConnect,
      onDisconnect: handleDisconnect,
      onError: handleError,
      onMessage: handleMessage,
    })
    await wsClient.connect()
  } catch (error) {
    connectionStatus.value = 'error'
    addSystemMessage(`连接失败: ${error instanceof Error ? error.message : '未知错误'}`, 'error')
  }
}

const disconnectWebSocket = () => {
  wsClient.disconnect()
}

// ========== 房间管理 ==========
const joinRoom = () => {
  if (!roomIdInput.value) return
  const success = wsClient.joinRoom(roomIdInput.value)
  if (success) {
    currentRoom.value = roomIdInput.value
    roomIdInput.value = ''
  }
}

const leaveRoom = (roomId: string) => {
  wsClient.leaveRoom(roomId)
  if (currentRoom.value === roomId) {
    currentRoom.value = null
  }
}

// ========== 消息发送 ==========
const sendChatMessage = () => {
  if (!chatInput.value || !currentRoom.value) return

  const success = wsClient.sendChatMessage(currentRoom.value, chatInput.value, replyToMessageId.value || undefined)

  if (success) {
    messages.value.push({
      id: Date.now().toString(),
      type: 'sent',
      content: chatInput.value,
      time: new Date().toLocaleTimeString(),
      sender: authStore.username,
      roomId: currentRoom.value,
    })
    chatInput.value = ''
    replyToMessageId.value = null
  }
}

const sendPing = () => {
  lastPingTime.value = Date.now()
  wsClient.ping()
}

// ========== 其他操作 ==========
const clearMessages = () => {
  messages.value = [
    {
      id: '0',
      type: 'system',
      content: '消息已清空',
      time: new Date().toLocaleTimeString(),
    },
  ]
}

const refreshOnlineUsers = () => {
  wsClient.getOnlineUsers()
}

// ========== 生命周期 ==========
onUnmounted(() => {
  wsClient.disconnect()
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <Wifi class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        WebSocket 测试
      </h1>
      <p class="page-subtitle">测试 WebSocket 实时通信功能</p>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-lg)">
      <!-- 左侧：连接控制 -->
      <n-card title="连接状态">
        <n-descriptions :columns="1" bordered>
          <n-descriptions-item label="连接状态">
            <n-tag :type="statusType">
              <template #icon>
                <component :is="isConnected ? Wifi : WifiOff" class="icon-sm" />
              </template>
              {{ statusText }}
            </n-tag>
            <n-spin v-if="isConnecting || isReconnecting" size="small" style="margin-left: 8px" />
          </n-descriptions-item>
          <n-descriptions-item label="当前用户">
            {{ authStore.username || '未登录' }}
          </n-descriptions-item>
          <n-descriptions-item label="延迟">
            {{ latency !== null ? `${latency}ms` : '-' }}
          </n-descriptions-item>
          <n-descriptions-item label="全局在线用户">
            <div style="display: flex; align-items: center; gap: var(--space-sm)">
              <span>{{ onlineUsers }}</span>
              <n-button size="tiny" text type="primary" @click="refreshOnlineUsers">
                <RefreshCw class="icon-sm" />
              </n-button>
            </div>
          </n-descriptions-item>
        </n-descriptions>

        <n-divider />

        <div class="action-group">
          <n-button type="primary" :disabled="isConnected || isConnecting" @click="connectWebSocket">
            <template #icon>
              <Play class="icon-sm" />
            </template>
            连接
          </n-button>
          <n-button type="error" :disabled="!isConnected" @click="disconnectWebSocket">
            <template #icon>
              <Square class="icon-sm" />
            </template>
            断开
          </n-button>
          <n-button :disabled="!isConnected" @click="sendPing">
            <template #icon>
              <RefreshCw class="icon-sm" />
            </template>
            Ping
          </n-button>
        </div>
      </n-card>

      <!-- 右侧：房间管理 -->
      <n-card title="房间管理">
        <n-space vertical>
          <n-input-group>
            <n-input v-model:value="roomIdInput" placeholder="输入房间ID" @keyup.enter="joinRoom" />
            <n-button type="primary" :disabled="!isConnected || !roomIdInput" @click="joinRoom">
              <template #icon>
                <LogIn class="icon-sm" />
              </template>
              加入
            </n-button>
          </n-input-group>

          <n-divider />

          <div class="form-section-title">已加入房间</div>
          <n-space v-if="joinedRooms.length > 0" wrap>
            <n-tag
              v-for="roomId in joinedRooms"
              :key="roomId"
              :type="currentRoom === roomId ? 'primary' : 'default'"
              closable
              @click="currentRoom = roomId"
              @close="leaveRoom(roomId)"
            >
              {{ roomId }}
            </n-tag>
          </n-space>
          <n-empty v-else description="尚未加入任何房间" size="small" />

          <div class="form-section-title" style="margin-top: var(--space-md)">当前房间</div>
          <n-tag v-if="currentRoom" type="primary" size="large">
            <MessageCircle class="icon-sm" style="margin-right: 4px" />
            {{ currentRoom }}
          </n-tag>
          <n-tag v-else type="default">未选择</n-tag>
        </n-space>
      </n-card>
    </div>

    <!-- 消息区域 -->
    <n-card title="消息收发" style="margin-top: var(--space-lg)">
      <template #header-extra>
        <n-button size="small" text type="error" @click="clearMessages">
          <template #icon>
            <Trash2 class="icon-sm" />
          </template>
          清空
        </n-button>
      </template>

      <div style="display: grid; grid-template-columns: 1fr 350px; gap: var(--space-lg)">
        <!-- 消息列表 -->
        <div
          style="
            background-color: var(--bg-secondary);
            border-radius: var(--radius-md);
            padding: var(--space-md);
            min-height: 350px;
            max-height: 450px;
            overflow-y: auto;
            border: 1px solid var(--border-light);
          "
        >
          <div
            v-for="msg in messages"
            :key="msg.id"
            style="margin-bottom: var(--space-sm); font-family: monospace; font-size: 13px"
          >
            <span style="color: var(--text-muted)">[{{ msg.time }}]</span>
            <n-tag
              size="tiny"
              :type="
                msg.type === 'sent'
                  ? 'info'
                  : msg.type === 'received'
                    ? 'success'
                    : 'default'
              "
              style="margin-right: var(--space-sm)"
            >
              {{ msg.type === 'sent' ? '发送' : msg.type === 'received' ? '接收' : '系统' }}
            </n-tag>
            <span v-if="msg.sender" style="color: var(--primary); margin-right: 4px">
              {{ msg.sender }}:
            </span>
            <span
              :style="{
                color:
                  msg.type === 'sent'
                    ? 'var(--info)'
                    : msg.type === 'received'
                      ? 'var(--success)'
                      : 'var(--warning)',
              }"
            >
              {{ msg.content }}
            </span>
            <span v-if="msg.roomId" style="color: var(--text-muted); margin-left: 4px">
              ({{ msg.roomId }})
            </span>
          </div>
        </div>

        <!-- 发送消息 -->
        <div style="display: flex; flex-direction: column; gap: var(--space-md)">
          <n-alert v-if="!currentRoom" type="warning" :show-icon="false" size="small">
            请先加入并选择一个房间
          </n-alert>

          <n-input
            v-model:value="chatInput"
            type="textarea"
            :rows="6"
            placeholder="输入消息内容..."
            :disabled="!isConnected || !currentRoom"
            @keyup.enter.ctrl="sendChatMessage"
          />

          <n-input
            v-if="replyToMessageId"
            v-model:value="replyToMessageId"
            placeholder="回复消息ID（可选）"
            size="small"
            :disabled="!isConnected"
          >
            <template #suffix>
              <n-button text size="tiny" @click="replyToMessageId = null"> 清除 </n-button>
            </template>
          </n-input>

          <n-button
            type="primary"
            :disabled="!isConnected || !currentRoom || !chatInput"
            @click="sendChatMessage"
          >
            <template #icon>
              <Send class="icon-sm" />
            </template>
            发送消息 (Ctrl+Enter)
          </n-button>

          <n-divider />

          <div class="form-section-title">快捷操作</div>
          <n-space wrap>
            <n-button size="small" :disabled="!isConnected" @click="wsClient.sendTyping(currentRoom!)">
              正在输入...
            </n-button>
            <n-button
              size="small"
              :disabled="!isConnected"
              @click="wsClient.sendStopTyping(currentRoom!)"
            >
              停止输入
            </n-button>
          </n-space>
        </div>
      </div>
    </n-card>
  </div>
</template>
