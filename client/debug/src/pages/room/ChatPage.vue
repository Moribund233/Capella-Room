<template>
  <div class="chat-page" :class="{ 'with-test-panel': showTestPanel }">
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
          <n-button size="small" @click="toggleTestPanel">
            <template #icon>
              <n-icon :component="TestTube" size="14" />
            </template>
            测试
          </n-button>
          <n-button size="small" @click="leaveRoom">
            离开
          </n-button>
        </n-space>
      </template>

      <!-- 测试面板浮层 -->
      <ChatTestPanel v-if="showTestPanel" :room-id="roomId" />

      <!-- 错误提示 -->
      <n-alert v-if="wsStore.lastError" type="error" :show-icon="false" style="margin-bottom: 16px;">
        {{ wsStore.lastError }}
      </n-alert>

      <!-- 正在输入提示 -->
      <div v-if="typingUsers.length > 0" class="typing-indicator">
        <n-text depth="3" class="typing-text">
          {{ typingText }}
        </n-text>
        <span class="typing-dots">
          <span></span>
          <span></span>
          <span></span>
        </span>
      </div>

      <!-- 消息列表 -->
      <div ref="messagesContainer" class="messages-container">
        <n-empty v-if="messages.length === 0" description="暂无消息，开始聊天吧！" />
        <div
          v-for="msg in messages"
          :key="msg.id"
          :class="['message-item', msg.type, { 'is-self': msg.sender?.id === currentUserId, 'is-deleted': msg.isDeleted }]"
          @contextmenu.prevent="handleContextMenu($event, msg)"
        >
          <div v-if="msg.type === 'system'" class="system-message">
            {{ msg.content }}
          </div>
          <template v-else>
            <div class="message-header">
              <n-avatar size="small">{{ msg.sender?.username?.charAt(0).toUpperCase() || '?' }}</n-avatar>
              <span class="sender-name">{{ msg.sender?.username || '未知用户' }}</span>
              <span class="message-time">{{ formatTime(msg.time) }}</span>
              <n-tag v-if="msg.isEdited" size="tiny" class="edited-tag">已编辑</n-tag>
            </div>
            <div class="message-content">{{ msg.content }}</div>
            <!-- 已读状态 -->
            <div v-if="msg.type === 'sent' && msg.readBy && msg.readBy.length > 0" class="read-status">
              <n-icon :component="CheckCheck" size="12" />
              <span>已读 {{ msg.readBy.length }}</span>
            </div>
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
            @input="handleInput"
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

    <!-- 右键菜单 -->
    <n-dropdown
      trigger="manual"
      :x="contextMenuX"
      :y="contextMenuY"
      :show="showContextMenu"
      :options="contextMenuOptions"
      @select="handleContextMenuSelect"
      @clickoutside="showContextMenu = false"
    />

    <!-- 编辑消息弹窗 -->
    <n-modal
      v-model:show="showEditModal"
      preset="dialog"
      title="编辑消息"
      positive-text="保存"
      negative-text="取消"
      @positive-click="confirmEdit"
      @negative-click="cancelEdit"
    >
      <n-input
        v-model:value="editContent"
        type="textarea"
        :rows="3"
        placeholder="编辑消息内容..."
      />
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Users, TestTube, CheckCheck } from 'lucide-vue-next'
import type { DropdownOption } from 'naive-ui'
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
  NInputGroup,
  NInput,
  NDrawer,
  NDrawerContent,
  NAlert,
  NIcon,
  NText,
  NDropdown,
  NModal,
  useMessage,
} from 'naive-ui'
import { useAuthStore, useWebSocketStore } from '@/store'
import { getRoom, getRoomMessages } from '@/api/room'
import type { ChatMessage } from '@/store/websocket'
import type { UserStatus } from '@/types/websocket'
import { useResponsive } from '@/composables/useResponsive'
import { ChatTestPanel } from '@/components/test'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const { isMobile } = useResponsive()
const message = useMessage()

const roomId = computed(() => {
  const id = route.params.id
  return id && typeof id === 'string' && id !== 'undefined' ? id : ''
})
const roomName = ref('聊天房间')
const inputMessage = ref('')
const messagesContainer = ref<HTMLDivElement | null>(null)
const showUsersDrawer = ref(false)
const showTestPanel = ref(false)

// 输入状态防抖
let typingTimeout: number | null = null
let isTypingSent = false

// 右键菜单
const showContextMenu = ref(false)
const contextMenuX = ref(0)
const contextMenuY = ref(0)
const selectedMessage = ref<ChatMessage | null>(null)

// 编辑弹窗
const showEditModal = ref(false)
const editContent = ref('')

const messages = computed(() =>
  wsStore.chatMessages.filter(m => m.roomId === roomId.value)
)

const onlineUsers = computed(() => wsStore.onlineUsers)
const currentUserId = computed(() => authStore.user?.id)
const typingUsers = computed(() => wsStore.getTypingUsers)

// 正在输入文本
const typingText = computed(() => {
  const users = typingUsers.value
  if (users.length === 0) return ''
  if (users.length === 1) return `${users[0]!.username} 正在输入`
  if (users.length === 2) return `${users[0]!.username} 和 ${users[1]!.username} 正在输入`
  return `${users.length} 人正在输入`
})

// 右键菜单选项
const contextMenuOptions = computed<DropdownOption[]>(() => {
  if (!selectedMessage.value) return []
  const isOwnMessage = selectedMessage.value.sender?.id === currentUserId.value
  const options: DropdownOption[] = []

  if (isOwnMessage && !selectedMessage.value.isDeleted) {
    options.push({
      label: '编辑消息',
      key: 'edit',
    })
    options.push({
      label: '删除消息',
      key: 'delete',
    })
  }

  options.push({
    label: '复制消息',
    key: 'copy',
  })

  return options
})

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

  // 发送停止输入
  if (isTypingSent) {
    wsStore.sendStopTyping(roomId.value)
    isTypingSent = false
  }
}

// 处理输入事件（发送正在输入状态）
function handleInput() {
  if (!wsStore.isConnected || !roomId.value) return

  // 清除之前的定时器
  if (typingTimeout) {
    clearTimeout(typingTimeout)
  }

  // 发送正在输入状态
  if (!isTypingSent) {
    wsStore.sendTyping(roomId.value)
    isTypingSent = true
  }

  // 3秒后发送停止输入
  typingTimeout = window.setTimeout(() => {
    if (isTypingSent) {
      wsStore.sendStopTyping(roomId.value)
      isTypingSent = false
    }
  }, 3000)
}

function leaveRoom() {
  wsStore.leaveRoom(roomId.value)
  router.push('/room/list')
}

function toggleTestPanel() {
  showTestPanel.value = !showTestPanel.value
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

// 监听 WebSocket 连接状态，连接成功后自动加入房间
const unwatchConnected = watchEffect(() => {
  if (wsStore.isConnected && roomId.value) {
    wsStore.joinRoom(roomId.value)
  }
})

// 右键菜单处理
function handleContextMenu(event: MouseEvent, msg: ChatMessage) {
  if (msg.type === 'system' || msg.isDeleted) return

  selectedMessage.value = msg
  contextMenuX.value = event.clientX
  contextMenuY.value = event.clientY
  showContextMenu.value = true
}

function handleContextMenuSelect(key: string) {
  showContextMenu.value = false

  if (!selectedMessage.value) return

  switch (key) {
    case 'edit':
      if (selectedMessage.value.sender?.id === currentUserId.value) {
        editContent.value = selectedMessage.value.content
        showEditModal.value = true
      }
      break
    case 'delete':
      if (selectedMessage.value.id && selectedMessage.value.sender?.id === currentUserId.value) {
        wsStore.deleteMessage(selectedMessage.value.id)
        message.success('消息已删除')
      }
      break
    case 'copy':
      navigator.clipboard.writeText(selectedMessage.value.content).then(() => {
        message.success('已复制到剪贴板')
      })
      break
  }
}

function confirmEdit() {
  if (selectedMessage.value?.id && editContent.value.trim()) {
    wsStore.editMessage(selectedMessage.value.id, editContent.value.trim())
    message.success('消息已编辑')
  }
  showEditModal.value = false
  selectedMessage.value = null
}

function cancelEdit() {
  showEditModal.value = false
  selectedMessage.value = null
  editContent.value = ''
}

onMounted(async () => {
  // 获取房间信息
  try {
    const room = await getRoom(roomId.value)
    roomName.value = room.name
  } catch (error) {
    console.error('获取房间信息失败:', error)
  }

  // 加载历史消息
  try {
    const response = await getRoomMessages(roomId.value, { page: 1, per_page: 50 })
    const historyMessages = response.messages.map(msg => {
      const messageType: 'sent' | 'received' = msg.sender?.id === currentUserId.value ? 'sent' : 'received'
      return {
        id: msg.id,
        type: messageType,
        content: msg.content,
        time: msg.created_at,
        sender: msg.sender ? {
          id: msg.sender.id,
          username: msg.sender.username,
          status: 'online' as const,
          avatar_url: msg.sender.avatar_url
        } : undefined,
        roomId: roomId.value
      }
    })
    // 将历史消息添加到 store
    wsStore.loadHistoryMessages(roomId.value, historyMessages)
  } catch (error) {
    console.error('加载历史消息失败:', error)
  }

  // 连接 WebSocket（如果未连接）
  if (!wsStore.isConnected) {
    wsStore.connect()
  }
})

onUnmounted(() => {
  // 停止监听
  unwatchConnected()
  // 清除输入定时器
  if (typingTimeout) {
    clearTimeout(typingTimeout)
  }
  // 离开房间
  wsStore.leaveRoom(roomId.value)
})
</script>

<style scoped>
.chat-page {
  display: flex;
  flex-direction: column;
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

.typing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--bg-default);
  border-radius: 8px;
  margin-bottom: 12px;
}

.typing-text {
  font-size: 13px;
}

.typing-dots {
  display: flex;
  gap: 3px;
}

.typing-dots span {
  width: 6px;
  height: 6px;
  background: var(--text-color-3);
  border-radius: 50%;
  animation: typing-bounce 1.4s infinite ease-in-out both;
}

.typing-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.typing-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes typing-bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
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
  padding: 8px;
  border-radius: 8px;
  transition: background 0.2s;
  cursor: context-menu;
}

.message-item:hover {
  background: var(--bg-container);
}

.message-item.is-self {
  text-align: right;
}

.message-item.is-self .message-header {
  justify-content: flex-end;
}

.message-item.is-deleted {
  opacity: 0.6;
}

.message-item.is-deleted .message-content {
  font-style: italic;
  color: var(--text-color-3);
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

.edited-tag {
  margin-left: 4px;
}

.message-content {
  padding: 8px 12px;
  background: var(--bg-container);
  border-radius: 8px;
  display: inline-block;
  max-width: 70%;
  word-break: break-word;
  text-align: left;
}

.message-item.is-self .message-content {
  background: var(--primary-color);
  color: var(--n-text-color, white);
}

.read-status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--success-color);
  margin-top: 4px;
  justify-content: flex-end;
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
  .chat-page {
    padding: 12px;
    gap: 12px;
    height: calc(100vh - 56px);
  }

  .mobile-header {
    display: flex;
    justify-content: flex-end;
    padding: 0 4px;
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

  .typing-indicator {
    padding: 6px 12px;
  }
}

/* 小屏幕手机适配 */
@media (max-width: 480px) {
  .chat-page {
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
