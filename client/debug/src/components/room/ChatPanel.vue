<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useMessage } from 'naive-ui'
import { Send, Trash2, Reply, LogIn, LogOut } from 'lucide-vue-next'
import { getRoomMessages, deleteMessage, type Message } from '@/api'
import { useWebSocketStore } from '@/stores/websocket'
import type { TestUser } from '@/utils/authUtils'

const props = defineProps<{
  roomId: string
  currentUser: TestUser | null
}>()

const emit = defineEmits<{
  (e: 'reply', message: Message): void
  (e: 'delete', messageId: string): void
}>()

const message = useMessage()
const wsStore = useWebSocketStore()

// ========== 状态 ==========
const messages = ref<Message[]>([])
const loading = ref(false)
const messageContent = ref('')
const replyToMessage = ref<Message | null>(null)
const messagesContainer = ref<HTMLElement | null>(null)
const hasMore = ref(false)

// 系统消息（成员加入/离开）
interface SystemMessage {
  id: string
  type: 'system'
  content: string
  time: string
}
const systemMessages = ref<SystemMessage[]>([])

// ========== 计算属性 ==========
const canSend = computed(() => {
  return messageContent.value.trim().length > 0 && props.roomId && wsStore.isConnected
})

// 判断消息是否是自己发送的
const isOwnMessage = (msg: Message): boolean => {
  const senderId = msg.sender?.id
  const currentUserId = props.currentUser?.id
  const isOwn = senderId === currentUserId
  return isOwn
}

// ========== 消息加载 ==========
const loadMessages = async () => {
  if (!props.roomId) return

  loading.value = true
  try {
    const result = await getRoomMessages(props.roomId, { limit: 50 })
    messages.value = result.messages
    hasMore.value = result.has_more
    await scrollToBottom()
  } catch (error) {
    message.error('加载消息失败')
    console.error('[ChatPanel] 加载消息错误:', error)
  } finally {
    loading.value = false
  }
}

const loadMoreMessages = async () => {
  if (!props.roomId || !hasMore.value || messages.value.length === 0) return

  try {
    const firstMessage = messages.value[0]
    if (!firstMessage) return
    const result = await getRoomMessages(props.roomId, {
      before: firstMessage.id,
      limit: 30,
    })

    if (result.messages.length > 0) {
      messages.value.unshift(...result.messages)
    }
    hasMore.value = result.has_more
  } catch (error) {
    console.error('[ChatPanel] 加载更多消息错误:', error)
  }
}

// ========== 发送消息 ==========
const handleSendMessage = async () => {
  if (!canSend.value) return

  try {
    const success = wsStore.send({
      type: 'ChatMessage',
      payload: {
        room_id: props.roomId,
        content: messageContent.value.trim(),
        reply_to: replyToMessage.value?.id || null,
      },
    })

    if (success) {
      messageContent.value = ''
      replyToMessage.value = null
      // 等待 WebSocket 推送新消息后刷新
      setTimeout(() => loadMessages(), 200)
    } else {
      message.error('发送消息失败')
    }
  } catch (error) {
    message.error('发送消息失败')
    console.error(error)
  }
}

// ========== 删除消息 ==========
const handleDeleteMessage = async (msg: Message) => {
  try {
    await deleteMessage(msg.id)
    message.success('消息已删除')
    emit('delete', msg.id)
    await loadMessages()
  } catch (error) {
    message.error('删除消息失败')
    console.error(error)
  }
}

// ========== 回复消息 ==========
const handleReply = (msg: Message) => {
  replyToMessage.value = msg
  emit('reply', msg)
}

const cancelReply = () => {
  replyToMessage.value = null
}

// ========== 滚动控制 ==========
const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

// ========== 格式化时间 ==========
const formatTime = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
}

// ========== 添加系统消息 ==========
const addSystemMessage = (content: string) => {
  systemMessages.value.push({
    id: Date.now().toString() + Math.random().toString(),
    type: 'system',
    content,
    time: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' }),
  })
  // 限制系统消息数量
  if (systemMessages.value.length > 50) {
    systemMessages.value = systemMessages.value.slice(-50)
  }
  scrollToBottom()
}

// ========== 监听新消息 ==========
let wsUnsubscribeNewMessage: (() => void) | null = null
let wsUnsubscribeUserJoined: (() => void) | null = null
let wsUnsubscribeUserLeft: (() => void) | null = null

onMounted(() => {
  loadMessages()

  // 注册消息处理器监听新消息
  wsUnsubscribeNewMessage = wsStore.onMessage('NewMessage', (payload: any) => {
    if (payload?.room_id === props.roomId) {
      setTimeout(() => loadMessages(), 100)
    }
  })

  // 监听用户加入房间
  wsUnsubscribeUserJoined = wsStore.onMessage('UserJoined', (payload: any) => {
    if (payload?.room_id === props.roomId) {
      addSystemMessage(`${payload.username} 加入了房间`)
    }
  })

  // 监听用户离开房间
  wsUnsubscribeUserLeft = wsStore.onMessage('UserLeft', (payload: any) => {
    if (payload?.room_id === props.roomId) {
      addSystemMessage(`${payload.username} 离开了房间`)
    }
  })
})

onUnmounted(() => {
  if (wsUnsubscribeNewMessage) {
    wsUnsubscribeNewMessage()
    wsUnsubscribeNewMessage = null
  }
  if (wsUnsubscribeUserJoined) {
    wsUnsubscribeUserJoined()
    wsUnsubscribeUserJoined = null
  }
  if (wsUnsubscribeUserLeft) {
    wsUnsubscribeUserLeft()
    wsUnsubscribeUserLeft = null
  }
})

// 监听房间变化
watch(() => props.roomId, () => {
  messages.value = []
  systemMessages.value = []
  replyToMessage.value = null
  loadMessages()
})
</script>

<template>
  <div class="chat-panel">
    <!-- 消息列表 -->
    <div ref="messagesContainer" class="messages-container">
      <div v-if="hasMore" class="load-more">
        <n-button text size="small" @click="loadMoreMessages">
          加载更多
        </n-button>
      </div>

      <div v-if="loading && messages.length === 0" class="loading-state">
        <n-spin size="medium" />
      </div>

      <div v-else-if="messages.length === 0 && systemMessages.length === 0" class="empty-state">
        <n-empty description="暂无消息" />
      </div>

      <!-- 系统消息（成员加入/离开） -->
      <div
        v-for="sysMsg in systemMessages"
        :key="sysMsg.id"
        class="system-message"
      >
        <n-divider dashed>
          <n-space align="center" size="small" class="system-message-content">
            <LogIn v-if="sysMsg.content.includes('加入')" :size="12" />
            <LogOut v-else :size="12" />
            <span class="system-text">{{ sysMsg.content }}</span>
            <span class="system-time">{{ sysMsg.time }}</span>
          </n-space>
        </n-divider>
      </div>

      <!-- 聊天消息 -->
      <div
        v-for="msg in messages"
        :key="msg.id"
        :class="['message-wrapper', { 'message-wrapper-own': isOwnMessage(msg) }]"
      >
        <div
          :class="['message-item', { 'message-own': isOwnMessage(msg) }]"
        >
          <!-- 回复引用 -->
          <div v-if="msg.reply_to" class="reply-reference">
            <Reply class="icon-xs" />
            <span class="reply-text">
              回复消息
            </span>
          </div>

          <div class="message-header">
            <n-avatar
              v-if="msg.sender?.avatar_url"
              size="small"
              :src="msg.sender.avatar_url"
            />
            <n-avatar v-else size="small" :style="{ backgroundColor: isOwnMessage(msg) ? '#52c41a' : 'var(--primary)' }">
              {{ msg.sender?.username?.charAt(0)?.toUpperCase() || '?' }}
            </n-avatar>
            <span class="sender-name">{{ msg.sender?.username || 'Unknown' }}</span>
            <span class="message-time">{{ formatTime(msg.created_at) }}</span>
            <n-space class="message-actions">
              <n-button size="tiny" text @click="handleReply(msg)">
                <template #icon>
                  <Reply class="icon-xs" />
                </template>
              </n-button>
              <n-button size="tiny" text type="error" @click="handleDeleteMessage(msg)">
                <template #icon>
                  <Trash2 class="icon-xs" />
                </template>
              </n-button>
            </n-space>
          </div>

          <div class="message-content">{{ msg.content }}</div>
        </div>
      </div>
    </div>

    <!-- 回复提示 -->
    <div v-if="replyToMessage" class="reply-indicator">
      <div class="reply-info">
        <Reply class="icon-xs" />
        <span>
          回复 {{ replyToMessage.sender?.username || 'Unknown' }}: {{ replyToMessage.content.substring(0, 30) }}{{ replyToMessage.content.length > 30 ? '...' : '' }}
        </span>
      </div>
      <n-button size="tiny" text @click="cancelReply">
        <template #icon>
          <Trash2 class="icon-xs" />
        </template>
      </n-button>
    </div>

    <!-- 发送区域 -->
    <div class="send-area">
      <n-input-group>
        <n-input
          v-model:value="messageContent"
          placeholder="输入消息内容..."
          :disabled="!wsStore.isConnected"
          @keyup.enter="handleSendMessage"
        />
        <n-button
          type="primary"
          :disabled="!canSend"
          @click="handleSendMessage"
        >
          <template #icon>
            <Send class="icon-sm" />
          </template>
          发送
        </n-button>
      </n-input-group>
      <div v-if="!wsStore.isConnected" class="connection-status">
        <n-text type="warning">WebSocket 未连接</n-text>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.load-more {
  text-align: center;
  padding: var(--space-sm);
}

.loading-state,
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 200px;
}

/* 系统消息（成员加入/离开） */
.system-message {
  display: flex;
  justify-content: center;
  align-items: center;
  margin: var(--space-xs) 0;
}

.system-message :deep(.n-divider) {
  margin: 0;
  color: var(--text-muted);
}

.system-message-content {
  font-size: 12px;
  color: var(--text-muted);
}

.system-text {
  color: var(--text-secondary);
}

.system-time {
  font-size: 11px;
  color: var(--text-muted);
}

/* 消息包装器 - 用于对齐 */
.message-wrapper {
  display: flex;
  justify-content: flex-start;
  width: 100%;
  margin-bottom: var(--space-sm);
}

.message-wrapper-own {
  justify-content: flex-end;
}

/* 消息项 */
.message-item {
  padding: var(--space-sm) var(--space-md);
  background-color: var(--bg-white);
  border-radius: var(--radius-md);
  max-width: 70%;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.message-own {
  background-color: #e6f7ff;
  border: 1px solid #91d5ff;
}

.message-header {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  margin-bottom: var(--space-xs);
}

.sender-name {
  font-weight: 500;
  font-size: 13px;
}

.message-time {
  font-size: 11px;
  color: var(--text-muted);
}

.message-actions {
  margin-left: auto;
  opacity: 0;
  transition: opacity 0.2s;
}

.message-item:hover .message-actions {
  opacity: 1;
}

.message-content {
  padding-left: 32px;
  color: var(--text-primary);
  word-break: break-word;
}

.message-own .message-content {
  padding-left: 0;
  padding-right: 32px;
}

.reply-reference {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-xs) var(--space-sm);
  background-color: var(--bg-secondary);
  border-radius: var(--radius-sm);
  margin-bottom: var(--space-xs);
  font-size: 12px;
  color: var(--text-muted);
}

.reply-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.reply-indicator {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-sm) var(--space-md);
  background-color: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
}

.reply-info {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: 13px;
  color: var(--text-muted);
}

.send-area {
  padding: var(--space-md);
  background-color: var(--bg-white);
  border-top: 1px solid var(--border-color);
  border-bottom-left-radius: var(--radius-md);
  border-bottom-right-radius: var(--radius-md);
}

.connection-status {
  margin-top: var(--space-xs);
  font-size: 12px;
}

.icon-xs {
  width: 14px;
  height: 14px;
}

.icon-sm {
  width: 16px;
  height: 16px;
}
</style>
