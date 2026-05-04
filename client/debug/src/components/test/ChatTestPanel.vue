<template>
  <div class="chat-test-panel">
    <!-- 面板内容 -->
    <div class="panel-content">
      <div class="panel-header">
        <h3 class="panel-title">测试用户控制</h3>
        <n-button text size="tiny" @click="refreshUsers">
          <template #icon>
            <n-icon :component="RefreshCw" size="14" />
          </template>
        </n-button>
      </div>

      <!-- 连接状态 -->
      <div class="connection-status">
        <n-tag :type="wsTestStore.connectedCount > 0 ? 'success' : 'warning'" size="small">
          {{ wsTestStore.connectedCount }} 个测试用户在线
        </n-tag>
      </div>

      <!-- 测试用户列表 -->
      <div class="users-list">
        <div
          v-for="user in connectedTestUsers"
          :key="user.userId"
          class="test-user-item"
          :class="{ active: selectedUserId === user.userId }"
          @click="selectUser(user.userId)"
        >
          <n-avatar size="small" class="user-avatar">
            {{ user.username.charAt(0).toUpperCase() }}
          </n-avatar>
          <div class="user-info">
            <span class="user-name">{{ user.username }}</span>
            <span class="user-status" :class="user.status">
              {{ getStatusText(user.status) }}
            </span>
          </div>
          <n-icon
            v-if="user.currentRoom === roomId"
            :component="CheckCircle2"
            size="16"
            class="in-room-icon"
          />
        </div>

        <n-empty v-if="connectedTestUsers.length === 0" description="无测试用户">
          <template #extra>
            <n-button text type="primary" size="small" @click="goToMultiUser">
              去创建
            </n-button>
          </template>
        </n-empty>
      </div>

      <!-- 操作区域 -->
      <n-divider style="margin: 12px 0" />

      <div class="actions-section">
        <n-space vertical size="small">
          <n-button
            size="small"
            :disabled="!selectedUserId || !roomId"
            @click="joinRoom"
          >
            <template #icon>
              <n-icon :component="LogIn" size="14" />
            </template>
            加入房间
          </n-button>

          <n-button
            size="small"
            :disabled="!selectedUserId || !roomId"
            @click="leaveRoom"
          >
            <template #icon>
              <n-icon :component="LogOut" size="14" />
            </template>
            离开房间
          </n-button>

          <n-button
            size="small"
            :disabled="!selectedUserId || !roomId"
            @click="showSendMessageModal = true"
          >
            <template #icon>
              <n-icon :component="MessageSquare" size="14" />
            </template>
            发送消息
          </n-button>

          <n-button
            size="small"
            :disabled="!selectedUserId || !roomId"
            @click="sendTyping"
          >
            <template #icon>
              <n-icon :component="Keyboard" size="14" />
            </template>
            正在输入
          </n-button>
        </n-space>
      </div>

      <!-- 批量操作 -->
      <n-divider style="margin: 12px 0" />

      <div class="batch-actions">
        <n-space vertical size="small">
          <n-button
            size="small"
            type="primary"
            ghost
            :disabled="connectedTestUsers.length === 0 || !roomId"
            @click="batchJoinRoom"
          >
            <template #icon>
              <n-icon :component="Users" size="14" />
            </template>
            全部加入房间
          </n-button>

          <n-button
            size="small"
            type="error"
            ghost
            :disabled="connectedTestUsers.length === 0"
            @click="batchLeaveRoom"
          >
            <template #icon>
              <n-icon :component="Users" size="14" />
            </template>
            全部离开房间
          </n-button>
        </n-space>
      </div>
    </div>

    <!-- 发送消息弹窗 -->
    <n-modal
      v-model:show="showSendMessageModal"
      preset="dialog"
      title="发送测试消息"
      positive-text="发送"
      negative-text="取消"
      @positive-click="sendMessage"
      @negative-click="showSendMessageModal = false"
    >
      <n-input
        v-model:value="messageContent"
        type="textarea"
        placeholder="输入消息内容..."
        :rows="3"
      />
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  NAvatar,
  NButton,
  NIcon,
  NTag,
  NSpace,
  NEmpty,
  NDivider,
  NModal,
  NInput,
  useMessage,
} from 'naive-ui'
import {
  RefreshCw,
  LogIn,
  LogOut,
  MessageSquare,
  Keyboard,
  Users,
  CheckCircle2,
} from 'lucide-vue-next'
import { useWsTestStore, type TestWsConnection } from '@/store/wsTest'

const props = defineProps<{
  roomId: string
}>()

const router = useRouter()
const message = useMessage()
const wsTestStore = useWsTestStore()

// 选中的用户
const selectedUserId = ref<string | null>(null)

// 发送消息弹窗
const showSendMessageModal = ref(false)
const messageContent = ref('')

// 获取已连接的测试用户
const connectedTestUsers = computed(() => {
  return wsTestStore.connectionList.filter((c: TestWsConnection) => c.status === 'authenticated')
})

// 获取状态文本
function getStatusText(status: TestWsConnection['status']): string {
  const statusMap: Record<string, string> = {
    connecting: '连接中',
    connected: '已连接',
    authenticated: '已认证',
    disconnected: '已断开',
    error: '错误',
  }
  return statusMap[status] || status
}

// 选择用户
function selectUser(userId: string) {
  selectedUserId.value = selectedUserId.value === userId ? null : userId
}

// 刷新用户列表
function refreshUsers() {
  message.info('已刷新')
}

// 跳转到多用户页面
function goToMultiUser() {
  router.push('/debug/multi-user')
}

// 加入房间
function joinRoom() {
  if (!selectedUserId.value || !props.roomId) return

  const conn = wsTestStore.connectionList.find((c: TestWsConnection) => c.userId === selectedUserId.value)
  if (conn?.ws) {
    conn.ws.send(JSON.stringify({
      type: 'JoinRoom',
      payload: { room_id: props.roomId }
    }))
    message.success('已发送加入房间请求')
  }
}

// 离开房间
function leaveRoom() {
  if (!selectedUserId.value || !props.roomId) return

  const conn = wsTestStore.connectionList.find((c: TestWsConnection) => c.userId === selectedUserId.value)
  if (conn?.ws) {
    conn.ws.send(JSON.stringify({
      type: 'LeaveRoom',
      payload: { room_id: props.roomId }
    }))
    message.success('已发送离开房间请求')
  }
}

// 发送消息
function sendMessage() {
  if (!selectedUserId.value || !props.roomId || !messageContent.value.trim()) {
    message.warning('请输入消息内容')
    return false
  }

  const conn = wsTestStore.connectionList.find((c: TestWsConnection) => c.userId === selectedUserId.value)
  if (conn?.ws) {
    conn.ws.send(JSON.stringify({
      type: 'ChatMessage',
      payload: {
        room_id: props.roomId,
        content: messageContent.value
      }
    }))
    message.success('消息已发送')
    messageContent.value = ''
    showSendMessageModal.value = false
  }
  return true
}

// 发送正在输入
function sendTyping() {
  if (!selectedUserId.value || !props.roomId) return

  const conn = wsTestStore.connectionList.find((c: TestWsConnection) => c.userId === selectedUserId.value)
  if (conn?.ws) {
    conn.ws.send(JSON.stringify({
      type: 'Typing',
      payload: { room_id: props.roomId }
    }))
    message.success('已发送正在输入状态')
  }
}

// 批量加入房间
function batchJoinRoom() {
  if (!props.roomId) return

  let count = 0
  connectedTestUsers.value.forEach((conn: TestWsConnection) => {
    if (conn.ws && conn.currentRoom !== props.roomId) {
      conn.ws.send(JSON.stringify({
        type: 'JoinRoom',
        payload: { room_id: props.roomId }
      }))
      count++
    }
  })
  message.success(`已发送 ${count} 个加入请求`)
}

// 批量离开房间
function batchLeaveRoom() {
  if (!props.roomId) return

  let count = 0
  connectedTestUsers.value.forEach((conn: TestWsConnection) => {
    if (conn.ws && conn.currentRoom === props.roomId) {
      conn.ws.send(JSON.stringify({
        type: 'LeaveRoom',
        payload: { room_id: props.roomId }
      }))
      count++
    }
  })
  message.success(`已发送 ${count} 个离开请求`)
}
</script>

<style scoped>
.chat-test-panel {
  position: absolute;
  left: 0;
  top: 56px;
  bottom: 0;
  width: 240px;
  background: var(--bg-container);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  z-index: 10;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.15);
}

.panel-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 12px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
}

.connection-status {
  margin-bottom: 12px;
}

.users-list {
  flex: 1;
  overflow-y: auto;
  min-height: 100px;
}

.test-user-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.test-user-item:hover {
  background: var(--bg-default);
}

.test-user-item.active {
  background: var(--primary-color-fade);
}

.user-avatar {
  flex-shrink: 0;
}

.user-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.user-name {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-status {
  font-size: 11px;
  color: var(--text-color-3);
}

.user-status.authenticated {
  color: var(--success-color);
}

.user-status.connecting {
  color: var(--warning-color);
}

.user-status.error {
  color: var(--error-color);
}

.in-room-icon {
  color: var(--success-color);
  flex-shrink: 0;
}

.actions-section,
.batch-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .chat-test-panel {
    width: 180px;
  }

  .panel-content {
    padding: 8px;
  }
}
</style>
