<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useWebSocketStore } from '@/stores/websocket'
import { useMessageStore } from '@/stores/message'
import { useAuthStore } from '@/stores/auth'
import { useWebSocket } from '@/composables/useWebSocket'
import { useMessageActions } from '@/composables/useMessageActions'
import { Search } from 'lucide-vue-next'
import ConnectionStatus from '@/components/chat/ConnectionStatus.vue'
import RoomDetail from '@/components/room/RoomDetail.vue'
import MessageList from '@/components/message/MessageList.vue'
import MessageInput from '@/components/message/MessageInput.vue'
import TypingIndicator from '@/components/message/TypingIndicator.vue'
import MessageSearch from '@/components/message/MessageSearch.vue'
import { WSMessageType } from '@/types/websocket'
import type {
  NewMessagePayload,
  MessageEditedPayload,
  MessageDeletedPayload,
  UserTypingPayload,
  UserStopTypingPayload,
  RoomMessageSummaryPayload,
} from '@/types/websocket'
import type { Message } from '@/types/message'

const route = useRoute()
const router = useRouter()
const roomStore = useRoomStore()
const wsStore = useWebSocketStore()
const messageStore = useMessageStore()
const authStore = useAuthStore()
const { currentRoom } = storeToRefs(roomStore)
const { isConnected } = storeToRefs(wsStore)
const { messages, loading, loadingMore, hasMore } = storeToRefs(messageStore)

// 初始化 WebSocket 连接
useWebSocket()

const roomId = ref(route.params.roomId as string)
const showDetail = ref(false)

// 使用消息操作组合式函数
const {
  replyingTo,
  editingMessage,
  typingUsers,
  showSearch,
  startReply,
  cancelReply,
  startEdit,
  cancelEdit,
  saveEdit,
  deleteMessage,
  closeSearch,
  jumpToMessage,
  addTypingUser,
  removeTypingUser,
} = useMessageActions(roomId.value)

// 加载房间信息 & 消息历史
async function loadRoom() {
  const id = route.params.roomId as string
  if (!id) return
  roomId.value = id
  messageStore.switchRoom(id)
  await Promise.all([
    roomStore.fetchRoomDetail(id),
    roomStore.fetchMembers(id),
    messageStore.fetchMessages(id),
  ])
  // 加入房间 WS 订阅（后端依赖此注册用户在内存中的订阅者列表）
  if (wsStore.isConnected) {
    wsStore.send('JoinRoom', { room_id: id })
  }
}

// 发送消息
function sendMessage(content: string) {
  if (!isConnected.value) return
  const replyToId = replyingTo.value?.id ?? null
  messageStore.sendMessage(roomId.value, content, replyToId)
  cancelReply()
}

// 处理编辑消息
function handleEditMessage(messageId: string, newContent: string) {
  saveEdit(messageId, newContent)
}

// 处理删除消息
function handleDeleteMessage(message: Message) {
  deleteMessage(message)
}

// 离开房间
async function handleLeave(leaveRoomId: string) {
  if (wsStore.isConnected) {
    wsStore.send('LeaveRoom', { room_id: leaveRoomId })
  }
  await roomStore.leaveRoom(leaveRoomId)
  router.push('/')
}

// WS 消息处理
function handleNewMessage(payload: NewMessagePayload) {
  messageStore.addIncomingMessage(payload)

  // 更新房间列表中的最新消息预览（当前房间不增加未读数）
  const isCurrentRoom = payload.room_id === roomId.value
  roomStore.updateRoomLastMessage(
    payload.room_id,
    {
      id: payload.message_id,
      content: payload.content,
      sender_name: payload.sender_name,
      created_at: payload.created_at,
    },
    !isCurrentRoom, // 只有非当前房间才增加未读数
  )
}

// 处理房间消息摘要（用于房间列表实时更新）
function handleRoomMessageSummary(payload: RoomMessageSummaryPayload) {
  // 更新房间列表中的最新消息预览
  const isCurrentRoom = payload.room_id === roomId.value
  roomStore.updateRoomLastMessage(
    payload.room_id,
    payload.last_message,
    !isCurrentRoom, // 只有非当前房间才增加未读数
  )
}

function handleMessageEdited(payload: MessageEditedPayload) {
  messageStore.handleMessageEdited(payload)
}

function handleMessageDeleted(payload: MessageDeletedPayload) {
  messageStore.handleMessageDeleted(payload)
}

function handleUserTyping(payload: UserTypingPayload) {
  if (payload.user_id !== authStore.user?.id) {
    addTypingUser(payload.user_id, payload.username)
  }
}

function handleUserStopTyping(payload: UserStopTypingPayload) {
  removeTypingUser(payload.user_id)
}

function handleError() {
  // 将当前所有正在发送的乐观消息标记为失败
  const pending = messages.value.filter((m) => m.sending)
  pending.forEach((m) => messageStore.failMessage(m.id))
}

function subscribeMessages() {
  wsStore.onMessage<NewMessagePayload>(WSMessageType.NEW_MESSAGE, handleNewMessage)
  wsStore.onMessage<MessageEditedPayload>(WSMessageType.MESSAGE_EDITED, handleMessageEdited)
  wsStore.onMessage<MessageDeletedPayload>(WSMessageType.MESSAGE_DELETED, handleMessageDeleted)
  wsStore.onMessage<UserTypingPayload>(WSMessageType.USER_TYPING, handleUserTyping)
  wsStore.onMessage<UserStopTypingPayload>(WSMessageType.USER_STOP_TYPING, handleUserStopTyping)
  wsStore.onMessage(WSMessageType.ERROR, handleError)
  wsStore.onMessage<RoomMessageSummaryPayload>(WSMessageType.ROOM_MESSAGE_SUMMARY, handleRoomMessageSummary)
}

function unsubscribeMessages() {
  wsStore.offMessage<NewMessagePayload>(WSMessageType.NEW_MESSAGE, handleNewMessage)
  wsStore.offMessage<MessageEditedPayload>(WSMessageType.MESSAGE_EDITED, handleMessageEdited)
  wsStore.offMessage<MessageDeletedPayload>(WSMessageType.MESSAGE_DELETED, handleMessageDeleted)
  wsStore.offMessage<UserTypingPayload>(WSMessageType.USER_TYPING, handleUserTyping)
  wsStore.offMessage<UserStopTypingPayload>(WSMessageType.USER_STOP_TYPING, handleUserStopTyping)
  wsStore.offMessage(WSMessageType.ERROR, handleError)
  wsStore.offMessage<RoomMessageSummaryPayload>(WSMessageType.ROOM_MESSAGE_SUMMARY, handleRoomMessageSummary)
}

function loadMore() {
  messageStore.fetchMore()
}

onMounted(() => {
  loadRoom()
  subscribeMessages()
})

// 路由变化时重新加载
watch(
  () => route.params.roomId,
  () => {
    showDetail.value = false
    loadRoom()
  },
)

// WS 重连后重新加入当前房间
watch(
  () => isConnected.value,
  (connected) => {
    if (connected && roomId.value) {
      wsStore.send('JoinRoom', { room_id: roomId.value })
    }
  },
)

// 移动端滑动手势
const chatRoomRef = ref<HTMLElement>()
let touchStartX = 0
let touchStartY = 0

function onTouchStart(e: TouchEvent) {
  const touch = e.touches[0]
  if (!touch) return
  touchStartX = touch.clientX
  touchStartY = touch.clientY
}

function onTouchEnd(e: TouchEvent) {
  const touch = e.changedTouches[0]
  if (!touch) return
  const touchEndX = touch.clientX
  const touchEndY = touch.clientY

  const deltaX = touchEndX - touchStartX
  const deltaY = touchEndY - touchStartY

  // 检测向右滑动（从屏幕左侧边缘开始）返回上一页
  if (deltaX > 80 && Math.abs(deltaY) < 50 && touchStartX < 50) {
    router.push('/')
  }
}

onUnmounted(() => {
  unsubscribeMessages()
  messageStore.$reset()
  roomStore.clearCurrentRoom()
})
</script>

<template>
  <div
    ref="chatRoomRef"
    class="chat-room"
    @touchstart="onTouchStart"
    @touchend="onTouchEnd"
  >
    <!-- 主内容区域 -->
    <div class="chat-room__main">
      <!-- 头部 -->
      <div class="chat-room__header">
        <div class="chat-room__header-left">
          <button class="chat-room__back-btn" @click="router.push('/')">←</button>
          <div class="chat-room__header-info">
            <h3 class="chat-room__title">{{ currentRoom?.name || '加载中...' }}</h3>
            <span v-if="currentRoom" class="chat-room__subtitle">
              {{ currentRoom.member_count }} 位成员
            </span>
          </div>
        </div>
        <div class="chat-room__header-right">
          <ConnectionStatus />
          <button
            class="chat-room__icon-btn"
            title="搜索消息"
            @click="showSearch = true"
          >
            <Search :size="18" />
          </button>
          <button
            class="chat-room__detail-btn"
            :class="{ 'chat-room__detail-btn--active': showDetail }"
            @click="showDetail = !showDetail"
          >
            详情
          </button>
        </div>
      </div>

      <!-- 消息列表 -->
      <MessageList
        :messages="messages"
        :loading="loading"
        :loading-more="loadingMore"
        :has-more="hasMore"
        :current-user-id="authStore.user?.id ?? ''"
        @load-more="loadMore"
        @reply="startReply"
        @edit="startEdit"
        @delete="handleDeleteMessage"
      />

      <!-- 正在输入提示 -->
      <TypingIndicator :typing-users="typingUsers" />

      <!-- 输入区域 -->
      <MessageInput
        :disabled="!isConnected"
        :reply-to="replyingTo"
        :editing-message="editingMessage"
        @send="sendMessage"
        @cancel-reply="cancelReply"
        @edit="handleEditMessage"
        @cancel-edit="cancelEdit"
      />
    </div>

    <!-- 房间详情侧栏 -->
    <RoomDetail
      :room-id="roomId"
      :visible="showDetail"
      @close="showDetail = false"
      @leave="handleLeave"
    />

    <!-- 消息搜索 -->
    <MessageSearch
      :room-id="roomId"
      :visible="showSearch"
      @close="closeSearch"
      @jump-to-message="jumpToMessage"
    />
  </div>
</template>

<style scoped>
.chat-room {
  display: flex;
  flex-direction: row;
  height: 100%;
  overflow: hidden;
}

.chat-room__main {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.chat-room__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  height: 52px;
  flex-shrink: 0;
}

.chat-room__header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.chat-room__back-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  padding: 4px;
  color: var(--color-text-secondary);
  display: none;
}

.chat-room__header-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.chat-room__title {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-room__subtitle {
  font-size: 11px;
  color: var(--color-text-tertiary, #999);
}

.chat-room__header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.chat-room__icon-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-white);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.chat-room__icon-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.chat-room__detail-btn {
  padding: 6px 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm, 6px);
  background: var(--color-white);
  color: var(--color-text-secondary);
  font-size: var(--font-size-small, 12px);
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
}

.chat-room__detail-btn:hover,
.chat-room__detail-btn--active {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
</style>
