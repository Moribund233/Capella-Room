<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useRoomStore } from '@/stores/room'
import { useMessageStore } from '@/stores/message'
import { useAuthStore } from '@/stores/auth'
import { useWebSocketStore } from '@/stores/websocket'
import {
  Search,
  Plus,
  Lock,
  ArrowRight,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()
const roomStore = useRoomStore()
const messageStore = useMessageStore()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()

// 加载状态
const loadingRooms = ref(false)
const loadingMessages = ref(false)

// 当前选中的房间ID
const activeRoomId = ref<string | null>(null)

// 当前房间
const activeRoom = computed(() => {
  if (!activeRoomId.value) return null
  return roomStore.roomMap.get(activeRoomId.value) || null
})

// 房间列表 - 按公开/私有分类
const publicRooms = computed(() => {
  return roomStore.rooms.filter(room => !room.is_private)
})

const privateRooms = computed(() => {
  return roomStore.rooms.filter(room => room.is_private)
})

// 消息列表
const messages = computed(() => messageStore.messages)

// 当前用户
const currentUser = computed(() => authStore.user)

// 输入框内容
const messageInput = ref('')

// 初始化
onMounted(async () => {
  // 连接WebSocket
  wsStore.connect()

  // 加载房间列表
  loadingRooms.value = true
  await roomStore.fetchMyRooms()
  loadingRooms.value = false

  // 如果有房间，默认选中第一个
  if (roomStore.rooms.length > 0 && !activeRoomId.value) {
    selectRoom(roomStore.rooms[0]!.id)
  }
})

// 监听房间变化，加载消息
watch(activeRoomId, async (newRoomId) => {
  if (newRoomId) {
    loadingMessages.value = true
    await messageStore.fetchMessages(newRoomId)
    loadingMessages.value = false

    // 加载房间成员
    await roomStore.fetchMembers(newRoomId)
  }
})

/**
 * 选择房间
 * @param roomId - 房间ID
 */
function selectRoom(roomId: string) {
  activeRoomId.value = roomId
  roomStore.currentRoom = roomStore.roomMap.get(roomId) || null
}

/**
 * 跳转到个人资料
 */
function goToProfile() {
  router.push('/profile')
}

/**
 * 发送消息
 */
async function sendMessage() {
  if (!messageInput.value.trim() || !activeRoomId.value) return

  const content = messageInput.value.trim()

  // 通过WebSocket发送消息
  wsStore.send('ChatMessage', {
    room_id: activeRoomId.value,
    content: content,
  })

  messageInput.value = ''
}

/**
 * 获取用户头像颜色
 * @param userId - 用户ID
 */
function getAvatarColor(userId: string): string {
  const colors: string[] = [
    'var(--accent)',
    'var(--accent-green)',
    'var(--accent-pink)',
    'var(--accent-blue)',
    'var(--accent-orange)',
  ]
  let hash = 0
  for (let i = 0; i < userId.length; i++) {
    hash = userId.charCodeAt(i) + ((hash << 5) - hash)
  }
  const index = Math.abs(hash) % colors.length
  return colors[index]!
}

/**
 * 获取用户首字母
 * @param name - 用户名
 */
function getInitials(name: string): string {
  return name ? name.charAt(0).toUpperCase() : '?'
}

/**
 * 格式化时间
 * @param date - 日期字符串
 */
function formatTime(date: string): string {
  return new Date(date).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="app-layout">
      <!-- 侧边栏 -->
      <aside class="sidebar">
        <div class="sidebar-header" @click="$router.push('/')">
          <span>CapellaRoom</span>
          <el-icon><ArrowRight /></el-icon>
        </div>

        <div class="sidebar-search">
          <el-input
            :placeholder="t('chat.findRoom')"
            :prefix-icon="Search"
            size="small"
          />
        </div>

        <div v-if="loadingRooms" class="rooms-loading">
          <el-skeleton :rows="5" animated />
        </div>

        <div v-else class="rooms">
          <!-- 公开房间 -->
          <div v-if="publicRooms.length > 0" class="room-category">
            <div class="category-header">
              <span>{{ t('chat.publicRooms') }}</span>
              <el-icon class="add-icon"><Plus /></el-icon>
            </div>
            <div
              v-for="room in publicRooms"
              :key="room.id"
              class="room"
              :class="{ active: activeRoomId === room.id }"
              @click="selectRoom(room.id)"
            >
              <span class="room-prefix">#</span>
              <span class="room-name">{{ room.name }}</span>
            </div>
          </div>

          <!-- 私有房间 -->
          <div v-if="privateRooms.length > 0" class="room-category">
            <div class="category-header">
              <span>{{ t('chat.myRooms') }}</span>
              <el-icon class="add-icon"><Plus /></el-icon>
            </div>
            <div
              v-for="room in privateRooms"
              :key="room.id"
              class="room"
              :class="{ active: activeRoomId === room.id }"
              @click="selectRoom(room.id)"
            >
              <span class="room-prefix">
                <el-icon><Lock /></el-icon>
              </span>
              <span class="room-name">{{ room.name }}</span>
            </div>
          </div>

          <!-- 没有房间时显示 -->
          <div v-if="publicRooms.length === 0 && privateRooms.length === 0" class="no-rooms">
            <el-empty :description="t('chat.noRooms')" />
          </div>
        </div>

        <!-- 用户信息 -->
        <div class="user-section" @click="goToProfile">
          <div class="user-avatar">
            <span>{{ getInitials(currentUser?.username || '') }}</span>
            <span class="status-dot"></span>
          </div>
          <div class="user-info">
            <div class="user-name">{{ currentUser?.username || 'User' }}</div>
            <div class="user-status">{{ t('chat.online') }}</div>
          </div>
        </div>
      </aside>

      <!-- 主内容区 -->
      <main class="main">
        <!-- 聊天头部 -->
        <header v-if="activeRoom" class="chat-header">
          <div class="room-info">
            <span class="room-hash">#</span>
            <span class="room-title">{{ activeRoom.name }}</span>
            <span v-if="activeRoom.description" class="room-topic">· {{ activeRoom.description }}</span>
          </div>
          <div class="chat-header-right">
            <span class="member-count">{{ roomStore.members.length }} {{ t('chat.members') }}</span>
          </div>
        </header>

        <!-- 消息列表 -->
        <div v-if="activeRoom" class="messages">
          <div v-if="loadingMessages" class="messages-loading">
            <el-skeleton :rows="10" animated />
          </div>

          <template v-else>
            <div
              v-for="message in messages"
              :key="message.id"
              class="message"
            >
              <div
                class="message-avatar"
                :style="{ background: getAvatarColor(message.sender.id) }"
              >
                {{ getInitials(message.sender.username) }}
              </div>
              <div class="message-body">
                <div class="message-header">
                  <span class="message-author">{{ message.sender.username }}</span>
                  <span class="message-time">{{ formatTime(message.created_at) }}</span>
                </div>
                <div class="message-content">{{ message.content }}</div>
              </div>
            </div>

            <div v-if="messages.length === 0" class="no-messages">
              <el-empty :description="t('chat.noMessages')" />
            </div>
          </template>
        </div>

        <!-- 未选择房间时显示 -->
        <div v-else class="no-room-selected">
          <el-empty :description="t('chat.selectRoom')" />
        </div>

        <!-- 输入框 -->
        <div v-if="activeRoom" class="input-area">
          <div class="input-wrapper">
            <el-input
              v-model="messageInput"
              type="textarea"
              :rows="1"
              :placeholder="t('chat.messagePlaceholder', { room: activeRoom.name })"
              resize="none"
              @keydown.enter.prevent="sendMessage"
            />
            <el-button
              type="primary"
              circle
              size="small"
              :disabled="!messageInput.trim()"
              @click="sendMessage"
            >
              <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>
        </div>
      </main>
    </div>
</template>

<style scoped lang="scss">
.app-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}

// 侧边栏
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  background: var(--sidebar-bg);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
}

.sidebar-header {
  height: var(--header-h);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;

  &:hover {
    background: var(--message-hover);
  }

  .el-icon {
    color: var(--muted);
  }
}

.sidebar-search {
  padding: 12px;

  :deep(.el-input__wrapper) {
    background-color: var(--bg);
  }
}

.rooms-loading {
  flex: 1;
  padding: 16px;
}

.rooms {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px;
}

.room-category {
  margin-bottom: 8px;
}

.category-header {
  padding: 16px 8px 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;

  &:hover {
    color: var(--fg);
  }
}

.add-icon {
  font-size: 12px;
}

.room {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: var(--radius);
  font-size: 15px;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.1s;

  &:hover {
    background: var(--message-hover);
    color: var(--fg);
  }

  &.active {
    background: var(--accent-soft);
    color: var(--fg);

    .room-prefix {
      color: var(--accent);
    }
  }
}

.room-prefix {
  color: var(--muted);
  opacity: 0.6;
  font-weight: 300;
  font-size: 16px;
  display: flex;
  align-items: center;

  .el-icon {
    font-size: 14px;
  }
}

.room-name {
  flex: 1;
}

.no-rooms {
  padding: 32px 16px;
}

// 用户信息
.user-section {
  border-top: 1px solid var(--border);
  padding: 10px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;

  &:hover {
    background: var(--message-hover);
  }
}

.user-avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  position: relative;
  flex-shrink: 0;
}

.status-dot {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--accent-green);
  border: 2px solid var(--sidebar-bg);
}

.user-info {
  flex: 1;
  min-width: 0;
}

.user-name {
  font-size: 14px;
  font-weight: 600;
}

.user-status {
  font-size: 12px;
  color: var(--muted);
}

// 主内容区
.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--bg);
}

.chat-header {
  height: var(--header-h);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.room-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
}

.room-hash {
  color: var(--muted);
  font-weight: 300;
}

.room-title {
  font-weight: 600;
}

.room-topic {
  color: var(--muted);
  font-size: 13px;
}

.chat-header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.member-count {
  font-size: 13px;
  color: var(--muted);
}

// 消息列表
.messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.messages-loading {
  padding: 20px;
}

.no-messages,
.no-room-selected {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.message {
  display: flex;
  gap: 12px;
  padding: 8px 0;

  &:hover {
    background: var(--message-hover);
  }
}

.message-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
}

.message-body {
  flex: 1;
  min-width: 0;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.message-author {
  font-weight: 600;
  font-size: 15px;
}

.message-time {
  font-size: 12px;
  color: var(--muted);
}

.message-content {
  font-size: 15px;
  line-height: 1.5;
  color: var(--fg);
  word-break: break-word;
}

// 输入框
.input-area {
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  background: var(--sidebar-bg);
  border-radius: var(--radius);
  padding: 8px 12px;
}

:deep(.el-input__wrapper) {
  background: transparent;
  box-shadow: none;
  padding: 0;
}

:deep(.el-textarea__inner) {
  background: transparent;
  border: none;
  resize: none;
  min-height: 36px;
  max-height: 200px;
}
</style>
