<template>
  <n-card title="WebSocket连接管理" class="ws-connection-card">
    <!-- 连接统计 -->
    <div class="connection-stats">
      <div class="stat-row">
        <n-statistic label="总连接" :value="stats.total" />
        <n-statistic label="已认证" :value="stats.authenticated" class="stat-success" />
        <n-statistic label="连接中" :value="stats.connected" class="stat-info" />
        <n-statistic label="错误" :value="stats.error" class="stat-error" />
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="action-buttons">
      <n-button
        type="primary"
        :loading="loading && currentOperation?.includes('连接')"
        :disabled="availableUsers.length === 0"
        @click="handleConnectAll"
      >
        <template #icon>
          <n-icon :component="Plug" />
        </template>
        连接全部
      </n-button>

      <n-button
        type="error"
        :disabled="stats.authenticated === 0"
        @click="handleDisconnectAll"
      >
        <template #icon>
          <n-icon :component="Unplug" />
        </template>
        断开全部
      </n-button>

      <n-button
        :disabled="stats.authenticated === 0"
        @click="showRoomModal = true"
      >
        <template #icon>
          <n-icon :component="DoorOpen" />
        </template>
        加入房间
      </n-button>
    </div>

    <!-- 用户连接列表 -->
    <div class="user-list">
      <n-divider>用户连接状态</n-divider>

      <n-empty v-if="testUsers.length === 0" description="暂无测试用户">
        <template #icon>
          <n-icon :component="Users" />
        </template>
        <template #extra>
          <n-text depth="3">请先在多用户测试页面创建用户</n-text>
        </template>
      </n-empty>

      <div v-else class="user-grid">
        <div
          v-for="user in testUsers"
          :key="user.id"
          class="user-item"
          :class="getConnectionClass(user.id)"
        >
          <div class="user-info">
            <n-avatar :size="32" :style="{ background: getStatusColor(user.id) }">
              {{ user.username.charAt(0).toUpperCase() }}
            </n-avatar>
            <div class="user-details">
              <div class="username">{{ user.username }}</div>
              <n-tag :type="getStatusType(user.id)" size="small">
                {{ getStatusText(user.id) }}
              </n-tag>
            </div>
          </div>

          <div class="user-actions">
            <n-button
              v-if="!isConnected(user.id) && user.isLoggedIn"
              size="small"
              type="primary"
              :loading="isConnecting(user.id)"
              @click="handleConnect(user)"
            >
              连接
            </n-button>
            <n-button
              v-else-if="isConnected(user.id)"
              size="small"
              type="error"
              @click="handleDisconnect(user.id)"
            >
              断开
            </n-button>
            <n-tag v-else size="small" type="default">未登录</n-tag>
          </div>
        </div>
      </div>
    </div>

    <!-- 房间选择模态框 -->
    <n-modal
      v-model:show="showRoomModal"
      title="选择房间"
      preset="card"
      style="width: 400px"
    >
      <n-form>
        <n-form-item label="房间">
          <n-select
            v-model:value="selectedRoomId"
            :options="roomOptions"
            placeholder="选择要加入的房间"
            clearable
          />
        </n-form-item>
        <n-form-item label="或输入房间ID">
          <n-input
            v-model:value="customRoomId"
            placeholder="输入房间UUID"
            clearable
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showRoomModal = false">取消</n-button>
          <n-button
            type="primary"
            :disabled="!targetRoomId"
            :loading="loading"
            @click="handleJoinRoom"
          >
            加入
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NCard,
  NStatistic,
  NButton,
  NIcon,
  NEmpty,
  NText,
  NAvatar,
  NTag,
  NDivider,
  NModal,
  NForm,
  NFormItem,
  NSelect,
  NInput,
  NSpace,
  useMessage,
} from 'naive-ui'
import { Plug, Unplug, DoorOpen, Users } from 'lucide-vue-next'
import { useWsTestStore, type WsConnectionStatus } from '@/store/wsTest'
import { useTestUsersStore, type TestUser } from '@/store/testUsers'
import { useWsTest } from '@/composables/test/useWsTest'

const message = useMessage()
const wsStore = useWsTestStore()
const testUsersStore = useTestUsersStore()
const wsTest = useWsTest()

// 本地状态
const showRoomModal = ref(false)
const selectedRoomId = ref<string | null>(null)
const customRoomId = ref('')

// 计算属性
const stats = computed(() => wsStore.connectionStats)
const loading = computed(() => wsTest.loading.value)
const currentOperation = computed(() => wsTest.currentOperation.value)
const testUsers = computed(() => testUsersStore.userList)
const availableUsers = computed(() => wsTest.availableUsers.value)

// 房间选项（示例，实际应从API获取）
const roomOptions = ref([
  { label: '测试房间1', value: '550e8400-e29b-41d4-a716-446655440001' },
  { label: '测试房间2', value: '550e8400-e29b-41d4-a716-446655440002' },
])

const targetRoomId = computed(() => customRoomId.value || selectedRoomId.value)

// 获取连接状态
function getConnectionStatus(userId: string): WsConnectionStatus {
  const conn = wsStore.connections.get(userId)
  return conn?.status || 'disconnected'
}

function isConnected(userId: string): boolean {
  const status = getConnectionStatus(userId)
  return status === 'authenticated'
}

function isConnecting(userId: string): boolean {
  const status = getConnectionStatus(userId)
  return status === 'connecting'
}

// 获取状态样式
function getConnectionClass(userId: string): string {
  const status = getConnectionStatus(userId)
  return `status-${status}`
}

function getStatusColor(userId: string): string {
  const status = getConnectionStatus(userId)
  const colors: Record<WsConnectionStatus, string> = {
    disconnected: '#999',
    connecting: '#fa0',
    connected: '#18a058',
    authenticated: '#2080f0',
    error: '#d03050',
  }
  return colors[status]
}

function getStatusType(userId: string): 'success' | 'error' | 'warning' | 'info' | 'default' {
  const status = getConnectionStatus(userId)
  const types: Record<WsConnectionStatus, 'success' | 'error' | 'warning' | 'info' | 'default'> = {
    disconnected: 'default',
    connecting: 'warning',
    connected: 'info',
    authenticated: 'success',
    error: 'error',
  }
  return types[status]
}

function getStatusText(userId: string): string {
  const status = getConnectionStatus(userId)
  const texts: Record<WsConnectionStatus, string> = {
    disconnected: '未连接',
    connecting: '连接中',
    connected: '已连接',
    authenticated: '已认证',
    error: '错误',
  }
  return texts[status]
}

// 操作处理
async function handleConnect(user: TestUser) {
  const success = await wsStore.connect(user)
  if (success) {
    message.success(`用户 ${user.username} 连接成功`)
  } else {
    message.error(`用户 ${user.username} 连接失败`)
  }
}

function handleDisconnect(userId: string) {
  wsStore.disconnect(userId)
  message.success('已断开连接')
}

async function handleConnectAll() {
  const result = await wsTest.batchConnect()
  if (result.success > 0) {
    message.success(`成功连接 ${result.success} 个用户`)
  }
  if (result.failed > 0) {
    message.error(`${result.failed} 个用户连接失败`)
  }
}

function handleDisconnectAll() {
  wsTest.batchDisconnect()
  message.success('已断开所有连接')
}

async function handleJoinRoom() {
  if (!targetRoomId.value) return

  const result = await wsTest.batchJoinRoom(targetRoomId.value)
  if (result.success > 0) {
    message.success(`${result.success} 个用户已加入房间`)
    showRoomModal.value = false
  }
}
</script>

<style scoped>
.ws-connection-card {
  margin-bottom: 16px;
}

.connection-stats {
  margin-bottom: 16px;
}

.stat-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-success :deep(.n-statistic__value) {
  color: #18a058;
}

.stat-info :deep(.n-statistic__value) {
  color: #2080f0;
}

.stat-error :deep(.n-statistic__value) {
  color: #d03050;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.user-list {
  margin-top: 16px;
}

.user-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.user-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 8px;
  background: var(--bg-default);
  border: 1px solid transparent;
  transition: all 0.3s ease;
}

.user-item.status-authenticated {
  border-color: #18a058;
  background: rgba(24, 160, 88, 0.05);
}

.user-item.status-connecting {
  border-color: #f0a020;
  background: rgba(240, 160, 32, 0.05);
}

.user-item.status-error {
  border-color: #d03050;
  background: rgba(208, 48, 80, 0.05);
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.username {
  font-weight: 500;
  font-size: 14px;
}

.user-actions {
  display: flex;
  gap: 8px;
}

@media (max-width: 640px) {
  .stat-row {
    grid-template-columns: repeat(2, 1fr);
  }

  .action-buttons {
    justify-content: stretch;
  }

  .action-buttons .n-button {
    flex: 1;
  }

  .user-grid {
    grid-template-columns: 1fr;
  }
}
</style>
