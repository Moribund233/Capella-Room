<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, shallowRef } from 'vue'
import { useMessage } from 'naive-ui'
import { Send, Trash2, Reply, LogIn, LogOut } from 'lucide-vue-next'
import { getRoomMessages, deleteMessage, type Message } from '@/api'
import { useWebSocketStore } from '@/stores/websocket'
import type { TestUser } from '@/utils/authUtils'

// 系统消息类型
interface SystemMessage {
  id: string
  type: 'system'
  content: string
  created_at: string
  system_type: 'join' | 'leave'
  username: string
}

// 消息项类型（普通消息或系统消息）
type MessageItem = Message | SystemMessage

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
const messages = shallowRef<MessageItem[]>([])
const loading = ref(false)
const messageContent = ref('')
const replyToMessage = ref<Message | null>(null)
const messagesContainer = ref<HTMLElement | null>(null)
const hasMore = ref(false)
const isAtBottom = ref(true)

// ========== 计算属性 ==========
const canSend = computed(() => {
  return messageContent.value.trim().length > 0 && props.roomId && wsStore.isConnected
})

// 判断消息是否是自己发送的
const isOwnMessage = (msg: Message): boolean => {
  return msg.sender?.id === props.currentUser?.id
}

// 判断消息项是否为系统消息
const isSystemMessage = (msg: MessageItem): msg is SystemMessage => {
  return msg.type === 'system'
}

// 按时间排序的消息（确保按时间正序：旧消息在前，新消息在后）
const sortedMessages = computed(() => {
  return [...messages.value].sort((a, b) => {
    return new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  })
})

// ========== 消息加载 ==========
const loadMessages = async () => {
  if (!props.roomId) return

  loading.value = true
  try {
    const result = await getRoomMessages(props.roomId, { limit: 100 })
    messages.value = result.messages
    hasMore.value = result.has_more
    if (isAtBottom.value) {
      await scrollToBottom()
    }
  } catch (error) {
    message.error('加载消息失败')
    console.error('[ChatPanel] 加载消息错误:', error)
  } finally {
    loading.value = false
  }
}

const loadMoreMessages = async () => {
  if (!props.roomId || !hasMore.value || messages.value.length === 0) return

  // 保存当前滚动位置
  const container = messagesContainer.value
  const oldScrollHeight = container?.scrollHeight || 0

  try {
    const firstMessage = sortedMessages.value[0]
    if (!firstMessage) return

    const result = await getRoomMessages(props.roomId, {
      before: firstMessage.id,
      limit: 50,
    })

    if (result.messages.length > 0) {
      messages.value = [...result.messages, ...messages.value]
      hasMore.value = result.has_more

      // 恢复滚动位置
      await nextTick()
      if (container) {
        const newScrollHeight = container.scrollHeight
        container.scrollTop = newScrollHeight - oldScrollHeight
      }
    }
  } catch (error) {
    console.error('[ChatPanel] 加载更多消息错误:', error)
  }
}

// ========== 发送消息 ==========
const handleSendMessage = async () => {
  if (!canSend.value) return

  const content = messageContent.value.trim()

  try {
    const success = wsStore.send({
      type: 'ChatMessage',
      payload: {
        room_id: props.roomId,
        content: content,
        reply_to: replyToMessage.value?.id || null,
      },
    })

    if (success) {
      messageContent.value = ''
      replyToMessage.value = null
      isAtBottom.value = true
      // 等待服务器响应后加载新消息
      setTimeout(() => loadMessages(), 100)
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
    messages.value = messages.value.filter(m => m.id !== msg.id)
    emit('delete', msg.id)
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
  const container = messagesContainer.value
  if (container) {
    container.scrollTop = container.scrollHeight
  }
}

const checkScrollPosition = () => {
  const container = messagesContainer.value
  if (!container) return

  const threshold = 50
  isAtBottom.value = container.scrollHeight - container.scrollTop - container.clientHeight <= threshold
}

// ========== 格式化时间 ==========
const formatTime = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
}

// ========== 处理新消息 ==========
const handleNewMessage = (payload: any) => {
  if (payload?.room_id !== props.roomId) return

  // 添加新消息到列表
  if (payload.message) {
    const newMessage = payload.message as Message
    // 检查是否已存在
    if (!messages.value.find(m => m.id === newMessage.id)) {
      messages.value = [...messages.value, newMessage]
      if (isAtBottom.value) {
        scrollToBottom()
      }
    }
  } else {
    // 如果没有完整消息数据，重新加载
    setTimeout(() => loadMessages(), 100)
  }
}

// ========== 处理系统消息（成员进入/离开） ==========
const handleUserJoined = (payload: any) => {
  if (payload?.room_id !== props.roomId) return

  const systemMsg: SystemMessage = {
    id: `system-join-${Date.now()}`,
    type: 'system',
    content: `${payload.username || 'Unknown'} 进入了房间`,
    created_at: new Date().toISOString(),
    system_type: 'join',
    username: payload.username || 'Unknown'
  }

  messages.value = [...messages.value, systemMsg]
  if (isAtBottom.value) {
    scrollToBottom()
  }
}

const handleUserLeft = (payload: any) => {
  if (payload?.room_id !== props.roomId) return

  const systemMsg: SystemMessage = {
    id: `system-leave-${Date.now()}`,
    type: 'system',
    content: `${payload.username || 'Unknown'} 离开了房间`,
    created_at: new Date().toISOString(),
    system_type: 'leave',
    username: payload.username || 'Unknown'
  }

  messages.value = [...messages.value, systemMsg]
  if (isAtBottom.value) {
    scrollToBottom()
  }
}

// ========== WebSocket 监听 ==========
let wsUnsubscribeNewMessage: (() => void) | null = null
let wsUnsubscribeUserJoined: (() => void) | null = null
let wsUnsubscribeUserLeft: (() => void) | null = null

onMounted(() => {
  loadMessages()

  // 注册消息处理器
  wsUnsubscribeNewMessage = wsStore.onMessage('NewMessage', handleNewMessage)
  wsUnsubscribeUserJoined = wsStore.onMessage('UserJoined', handleUserJoined)
  wsUnsubscribeUserLeft = wsStore.onMessage('UserLeft', handleUserLeft)
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
  replyToMessage.value = null
  isAtBottom.value = true
  loadMessages()
})
</script>

<template>
  <div class="chat-panel">
    <!-- 消息列表 -->
    <div
      ref="messagesContainer"
      class="messages-container"
      @scroll="checkScrollPosition"
    >
      <!-- 加载更多 -->
      <div v-if="hasMore" class="load-more">
        <n-button text size="small" @click="loadMoreMessages">
          加载更多历史消息
        </n-button>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading && messages.length === 0" class="loading-state">
        <n-spin size="medium" />
      </div>

      <!-- 空状态 -->
      <div v-else-if="messages.length === 0" class="empty-state">
        <n-empty description="暂无消息，开始聊天吧" />
      </div>

      <!-- 消息列表 -->
      <template v-else>
        <div
          v-for="msg in sortedMessages"
          :key="msg.id"
          :class="[
            'message-item',
            { 'message-own': !isSystemMessage(msg) && isOwnMessage(msg) },
            { 'message-system': isSystemMessage(msg) }
          ]"
        >
          <!-- 系统消息 -->
          <template v-if="isSystemMessage(msg)">
            <div class="system-message-content">
              <component
                :is="msg.system_type === 'join' ? LogIn : LogOut"
                class="icon-xs"
              />
              <span class="system-text">{{ msg.content }}</span>
              <span class="message-time">{{ formatTime(msg.created_at) }}</span>
            </div>
          </template>

          <!-- 普通消息 -->
          <template v-else>
            <!-- 回复引用 -->
            <div v-if="msg.reply_to" class="reply-reference">
              <Reply class="icon-xs" />
              <span class="reply-text">回复消息</span>
            </div>

            <!-- 消息头部 -->
            <div class="message-header">
              <n-avatar
                v-if="msg.sender?.avatar_url"
                size="small"
                :src="msg.sender.avatar_url"
              />
              <n-avatar
                v-else
                size="small"
                :style="{ backgroundColor: isOwnMessage(msg) ? '#52c41a' : 'var(--primary-color)' }"
              >
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

            <!-- 消息内容 -->
            <div class="message-content">{{ msg.content }}</div>
          </template>
        </div>
      </template>
    </div>

    <!-- 回复提示 -->
    <div v-if="replyToMessage" class="reply-indicator">
      <div class="reply-info">
        <Reply class="icon-xs" />
        <span>
          回复 {{ replyToMessage.sender?.username || 'Unknown' }}:
          {{ replyToMessage.content.substring(0, 30) }}{{ replyToMessage.content.length > 30 ? '...' : '' }}
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
  border: 1px solid var(--border-color);
  overflow: hidden;
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
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

/* 消息项 */
.message-item {
  padding: var(--space-sm) var(--space-md);
  background-color: var(--bg-white);
  border-radius: var(--radius-md);
  max-width: 80%;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  align-self: flex-start;
}

.message-own {
  background-color: #e6f7ff;
  border: 1px solid #91d5ff;
  align-self: flex-end;
}

/* 系统消息 */
.message-system {
  align-self: center;
  max-width: 90%;
  padding: var(--space-xs) var(--space-lg);
  background-color: transparent;
  box-shadow: none;
  border: none;
}

.system-message-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  padding: var(--space-xs) var(--space-md);
  background-color: var(--bg-secondary);
  border-radius: var(--radius-lg);
  font-size: 13px;
  color: var(--text-secondary);
}

.system-message-content .icon-xs {
  color: var(--primary-color);
}

.system-text {
  font-weight: 500;
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
  color: var(--text-primary);
  word-break: break-word;
  line-height: 1.5;
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
  flex-shrink: 0;
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
