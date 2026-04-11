<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { useAuthStore } from '@/stores/auth'
import { storeToRefs } from 'pinia'
import { getMyRooms, type Room } from '@/api'
import { useMessage } from 'naive-ui'
import {
  Wifi,
  WifiOff,
  Send,
  Play,
  Square,
  Trash2,
  MessageSquare,
  RefreshCw,
  LogIn,
  LogOut,
  MessageCircle,
  List,
} from 'lucide-vue-next'

const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const message = useMessage()
const {
  status,
  isConnected,
  isConnecting,
  joinedRooms,
  currentRoom: storeCurrentRoom,
  chatMessages,
  onlineUsers,
  latency,
  reconnectAttempts
} = storeToRefs(wsStore)

// ========== 本地状态 ==========
const roomIdInput = ref('')
const chatInput = ref('')
const replyToMessageId = ref<string | null>(null)
const myRooms = ref<Room[]>([])
const showRoomSelector = ref(false)
const localMessages = ref<Array<{
  id: string
  type: 'system' | 'sent' | 'received'
  content: string
  time: string
  sender?: string
  roomId?: string
}>>([
  {
    id: '0',
    type: 'system',
    content: 'WebSocket 测试页面已加载',
    time: new Date().toLocaleTimeString(),
  },
])

// 同步 store 的 currentRoom
const currentRoom = computed({
  get: () => storeCurrentRoom.value,
  set: (val) => { storeCurrentRoom.value = val }
})

// ========== 计算属性 ==========
const isReconnecting = computed(() => status.value === 'reconnecting')

const statusText = computed(() => {
  switch (status.value) {
    case 'connected':
      return '已连接'
    case 'connecting':
      return '连接中...'
    case 'reconnecting':
      return `重连中(${reconnectAttempts.value})`
    case 'disconnected':
      return '未连接'
    default:
      return '未知'
  }
})

const statusType = computed(() => {
  switch (status.value) {
    case 'connected':
      return 'success'
    case 'connecting':
    case 'reconnecting':
      return 'warning'
    case 'disconnected':
      return 'error'
    default:
      return 'default'
  }
})

// ========== 消息辅助函数 ==========
const addSystemMessage = (content: string, level: 'info' | 'error' = 'info', roomId?: string) => {
  localMessages.value.push({
    id: Date.now().toString(),
    type: 'system',
    content,
    time: new Date().toLocaleTimeString(),
    roomId,
  })
}

// 监听 store 中的聊天消息
watch(chatMessages, (newMessages) => {
  // 将 store 的消息同步到本地显示
  newMessages.forEach(msg => {
    const exists = localMessages.value.some(m => m.id === msg.id)
    if (!exists) {
      localMessages.value.push(msg)
    }
  })
}, { deep: true })

// ========== 连接控制 ==========
const connectWebSocket = () => {
  wsStore.connect()
}

const disconnectWebSocket = () => {
  wsStore.disconnect()
  addSystemMessage('WebSocket 连接已断开')
}

// ========== 房间管理 ==========
const joinRoom = () => {
  if (!roomIdInput.value) return
  const success = wsStore.joinRoom(roomIdInput.value)
  if (success) {
    currentRoom.value = roomIdInput.value
    roomIdInput.value = ''
    addSystemMessage(`已加入房间: ${currentRoom.value}`)
  }
}

const leaveRoom = (roomId: string) => {
  wsStore.leaveRoom(roomId)
  if (currentRoom.value === roomId) {
    currentRoom.value = null
  }
  addSystemMessage(`已离开房间: ${roomId}`)
}

// ========== 消息发送 ==========
const sendChatMessage = () => {
  if (!chatInput.value || !currentRoom.value) return

  const success = wsStore.sendChatMessage(
    currentRoom.value,
    chatInput.value,
    replyToMessageId.value || undefined
  )

  if (success) {
    // 添加到本地消息
    localMessages.value.push({
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
  wsStore.ping()
}

// ========== 其他操作 ==========
const clearMessages = () => {
  localMessages.value = [
    {
      id: '0',
      type: 'system',
      content: '消息已清空',
      time: new Date().toLocaleTimeString(),
    },
  ]
  wsStore.clearChatMessages()
}

const refreshOnlineUsers = () => {
  wsStore.getOnlineUsers()
}

// ========== 房间列表 ==========
const loadMyRooms = async () => {
  try {
    myRooms.value = await getMyRooms()
    showRoomSelector.value = true
  } catch (error) {
    message.error('加载房间列表失败')
    console.error(error)
  }
}

const selectRoom = (room: Room) => {
  // 如果房间还没加入，先加入
  if (!joinedRooms.value.includes(room.id)) {
    const success = wsStore.joinRoom(room.id)
    if (success) {
      addSystemMessage(`已加入房间: ${room.name} (${room.id})`)
    }
  }
  currentRoom.value = room.id
  showRoomSelector.value = false
  message.success(`已选择房间: ${room.name}`)
}

// ========== 生命周期 ==========
onMounted(() => {
  // 如果未连接，自动连接
  if (!isConnected.value) {
    wsStore.connect()
  }
})

onUnmounted(() => {
  // 注意：不在组件卸载时断开连接，因为它是全局共享的
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
            <n-button :disabled="!isConnected" @click="loadMyRooms">
              <template #icon>
                <List class="icon-sm" />
              </template>
              从列表选择
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
            v-for="msg in localMessages"
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
            <n-button size="small" :disabled="!isConnected || !currentRoom" @click="wsStore.sendTyping(currentRoom!)">
              正在输入...
            </n-button>
            <n-button
              size="small"
              :disabled="!isConnected || !currentRoom"
              @click="wsStore.sendStopTyping(currentRoom!)"
            >
              停止输入
            </n-button>
          </n-space>
        </div>
      </div>
    </n-card>

    <!-- 房间选择器弹窗 -->
    <n-modal
      v-model:show="showRoomSelector"
      title="选择房间"
      preset="card"
      style="width: 500px"
    >
      <n-empty v-if="myRooms.length === 0" description="暂无已加入的房间" />
      <n-list v-else bordered>
        <n-list-item
          v-for="room in myRooms"
          :key="room.id"
          clickable
          @click="selectRoom(room)"
        >
          <n-thing :title="room.name" :description="room.id">
            <template #description>
              <n-space vertical size="small">
                <n-text type="info" style="font-size: 12px">ID: {{ room.id }}</n-text>
                <n-text depth="3" style="font-size: 12px">{{ room.description || '无描述' }}</n-text>
              </n-space>
            </template>
            <template #avatar>
              <n-tag :type="room.is_private ? 'warning' : 'success'" size="small">
                {{ room.is_private ? '私有' : '公开' }}
              </n-tag>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-modal>
  </div>
</template>
