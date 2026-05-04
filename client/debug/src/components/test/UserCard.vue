<template>
  <div class="user-card" :class="cardClass">
    <div class="card-header">
      <div class="user-avatar">
        {{ avatarText }}
      </div>
      <div class="user-info">
        <div class="username">{{ user.username }}</div>
        <div class="user-id">ID: {{ shortId }}</div>
      </div>
      <div class="status-badge" :class="statusClass">
        {{ statusText }}
      </div>
    </div>

    <div class="card-body">
      <div class="info-row">
        <span class="label">创建</span>
        <span class="value">{{ formatTime(user.createdAt) }}</span>
      </div>
      <div v-if="user.tokenExpiry" class="info-row">
        <span class="label">过期</span>
        <span :class="['value', tokenExpiryClass]">{{ formatTime(user.tokenExpiry) }}</span>
      </div>
    </div>

    <div class="card-actions">
      <template v-if="!user.isLoggedIn">
        <n-button size="tiny" :loading="isLoading('login')" @click="handleLogin">
          <template #icon><n-icon :component="LogIn" /></template>
          登录
        </n-button>
      </template>
      <template v-else>
        <n-button
          v-if="!user.isConnected"
          size="tiny"
          :loading="isLoading('connect')"
          @click="handleConnect"
        >
          <template #icon><n-icon :component="Wifi" /></template>
          连WS
        </n-button>
        <n-button v-else size="tiny" @click="handleDisconnect">
          <template #icon><n-icon :component="WifiOff" /></template>
          断WS
        </n-button>

        <n-button size="tiny" :loading="isLoading('refresh')" @click="handleRefresh">
          <template #icon><n-icon :component="RefreshCw" /></template>
          刷新
        </n-button>

        <n-button size="tiny" :loading="isLoading('logout')" @click="handleLogout">
          <template #icon><n-icon :component="LogOut" /></template>
          登出
        </n-button>
      </template>

      <n-popconfirm @positive-click="handleDelete">
        <template #trigger>
          <n-button size="tiny" type="error" ghost>
            <template #icon><n-icon :component="Trash2" /></template>
          </n-button>
        </template>
        确定删除此用户吗？
      </n-popconfirm>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { NButton, NIcon, NPopconfirm, useMessage } from 'naive-ui'
import { LogIn, LogOut, Wifi, WifiOff, RefreshCw, Trash2 } from 'lucide-vue-next'
import type { TestUser } from '@/store/testUsers'
import {
  testUserLogin,
  testRefreshToken,
  testLogout,
  isTokenExpiringSoon as checkTokenExpiringSoon,
  parseTokenExpiry,
} from '@/api/test'
import { useTestUsersStore } from '@/store/testUsers'

const props = defineProps<{
  user: TestUser
}>()

const emit = defineEmits<{
  delete: [userId: string]
}>()

const message = useMessage()
const store = useTestUsersStore()

const loadingActions = ref<Set<string>>(new Set())

function isLoading(action: string): boolean {
  return loadingActions.value.has(`${action}_${props.user.id}`)
}

function setLoading(action: string, value: boolean) {
  const key = `${action}_${props.user.id}`
  if (value) {
    loadingActions.value.add(key)
  } else {
    loadingActions.value.delete(key)
  }
}

const avatarText = computed(() => {
  return props.user.username?.charAt(0)?.toUpperCase() || '?'
})

const shortId = computed(() => {
  return props.user.id.slice(0, 6)
})

const cardClass = computed(() => ({
  'is-online': props.user.isLoggedIn,
  'is-connected': props.user.isConnected,
}))

const statusClass = computed(() => ({
  'status-online': props.user.isConnected,
  'status-logged': props.user.isLoggedIn && !props.user.isConnected,
  'status-offline': !props.user.isLoggedIn,
}))

const statusText = computed(() => {
  if (props.user.isConnected) return 'WS'
  if (props.user.isLoggedIn) return '在线'
  return '离线'
})

const isTokenExpiringSoon = computed((): boolean => {
  if (!props.user.tokenExpiry) return false
  return checkTokenExpiringSoon(props.user.accessToken, 5)
})

const tokenExpiryClass = computed(() => ({
  'text-warning': isTokenExpiringSoon.value,
}))

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function handleLogin() {
  setLoading('login', true)
  try {
    const response = await testUserLogin(props.user.username, 'Test123456!')
    if (response) {
      store.updateUserLoginStatus(props.user.id, true, {
        accessToken: response.access_token,
        refreshToken: response.refresh_token,
        expiry: parseTokenExpiry(response.access_token),
      })
      message.success('登录成功')
    } else {
      message.error('登录失败')
    }
  } finally {
    setLoading('login', false)
  }
}

async function handleConnect() {
  setLoading('connect', true)
  try {
    await new Promise((resolve) => setTimeout(resolve, 500))
    store.updateUserConnectionStatus(props.user.id, true)
    message.success('WebSocket 连接成功')
  } finally {
    setLoading('connect', false)
  }
}

function handleDisconnect() {
  store.updateUserConnectionStatus(props.user.id, false)
  message.success('WebSocket 已断开')
}

async function handleRefresh() {
  setLoading('refresh', true)
  try {
    const response = await testRefreshToken(props.user.refreshToken)
    if (response) {
      store.updateUserToken(
        props.user.id,
        response.access_token,
        response.refresh_token,
        parseTokenExpiry(response.access_token)
      )
      message.success('Token 刷新成功')
    } else {
      message.error('Token 刷新失败')
      store.updateUserLoginStatus(props.user.id, false)
    }
  } finally {
    setLoading('refresh', false)
  }
}

async function handleLogout() {
  setLoading('logout', true)
  try {
    if (props.user.isConnected) {
      handleDisconnect()
    }
    await testLogout(props.user.accessToken)
    store.updateUserLoginStatus(props.user.id, false)
    message.success('登出成功')
  } finally {
    setLoading('logout', false)
  }
}

function handleDelete() {
  emit('delete', props.user.id)
}
</script>

<style scoped>
.user-card {
  background: var(--bg-container);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 12px;
  transition: all 0.2s ease;
}

.user-card.is-online {
  border-left: 3px solid #2080f0;
}

.user-card.is-connected {
  border-left: 3px solid #18a058;
}

/* 头部 */
.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.user-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
  font-size: 14px;
  flex-shrink: 0;
}

.user-info {
  flex: 1;
  min-width: 0;
}

.username {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-id {
  font-size: 10px;
  color: var(--text-secondary);
  font-family: monospace;
}

.status-badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 500;
  flex-shrink: 0;
}

.status-online {
  background: rgba(24, 160, 88, 0.15);
  color: #18a058;
}

.status-logged {
  background: rgba(32, 128, 240, 0.15);
  color: #2080f0;
}

.status-offline {
  background: var(--bg-default);
  color: var(--text-secondary);
}

/* 主体 */
.card-body {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-color);
}

.info-row {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.label {
  color: var(--text-secondary);
}

.value {
  color: var(--text-primary);
  font-family: monospace;
}

.text-warning {
  color: #f0a020;
}

/* 操作按钮 */
.card-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.card-actions :deep(.n-button) {
  padding: 0 8px;
}

/* 桌面端适配 */
@media (min-width: 768px) {
  .user-card {
    padding: 16px;
    border-radius: 12px;
  }

  .card-header {
    gap: 12px;
    margin-bottom: 12px;
  }

  .user-avatar {
    width: 40px;
    height: 40px;
    font-size: 16px;
  }

  .username {
    font-size: 15px;
  }

  .user-id {
    font-size: 11px;
  }

  .status-badge {
    padding: 3px 10px;
    font-size: 11px;
  }

  .card-body {
    gap: 6px;
    margin-bottom: 12px;
    padding-bottom: 12px;
  }

  .info-row {
    font-size: 12px;
  }

  .card-actions {
    gap: 8px;
  }

  .card-actions :deep(.n-button) {
    padding: 0 12px;
  }
}
</style>
