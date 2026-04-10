<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  LayoutDashboard,
  Wifi,
  Plug,
  MessageSquare,
  Users,
  Settings,
  TestTube,
  LogOut,
  User,
  CheckCircle
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const menuItems = [
  { key: 'dashboard', label: '仪表盘', icon: LayoutDashboard, path: '/' },
  { key: 'websocket', label: 'WebSocket测试', icon: Wifi, path: '/websocket' },
  { key: 'api', label: 'API测试', icon: Plug, path: '/api' },
  { key: 'rooms', label: '房间管理', icon: MessageSquare, path: '/rooms' },
  { key: 'users', label: '用户管理', icon: Users, path: '/users' },
  { key: 'messages', label: '消息测试', icon: TestTube, path: '/messages' },
  { key: 'e2e', label: '端到端测试', icon: CheckCircle, path: '/e2e' }
]

const activeKey = computed(() => {
  const item = menuItems.find((item) => item.path === route.path)
  return item?.key || 'dashboard'
})

const handleMenuClick = (key: string) => {
  const item = menuItems.find((item) => item.key === key)
  if (item) {
    router.push(item.path)
  }
}

const handleLogout = async () => {
  try {
    await authStore.logout()
    // 使用 replace 而不是 push，避免浏览器历史记录问题
    router.replace('/login')
  } catch (err) {
    console.error('登出失败:', err)
    // 即使登出失败，也强制跳转到登录页
    router.replace('/login')
  }
}
</script>

<template>
  <div class="sidebar">
    <div class="logo">
      <TestTube class="icon-md" />
      <span class="logo-text">Seredeli Debug</span>
    </div>

    <div class="menu-wrapper">
      <n-menu
        :value="activeKey"
        :options="
          menuItems.map((item) => ({
            key: item.key,
            label: item.label,
            icon: () =>
              h(item.icon, {
                class: 'icon-md'
              })
          }))
        "
        @update:value="handleMenuClick"
      />
    </div>

    <!-- 用户信息区域 -->
    <div class="user-section">
      <div class="user-info">
        <n-avatar size="small" :style="{ backgroundColor: 'var(--primary)' }">
          {{ authStore.userAvatar }}
        </n-avatar>
        <div class="user-details">
          <div class="username">{{ authStore.username }}</div>
          <div class="user-role">{{ authStore.roleText }}</div>
        </div>
      </div>
      <n-button text type="error" size="small" @click="handleLogout">
        <template #icon>
          <LogOut class="icon-sm" />
        </template>
        退出
      </n-button>
    </div>
  </div>
</template>

<script lang="ts">
import { h } from 'vue'
export { h }
</script>

<style scoped>
.sidebar {
  height: 100vh;
  background-color: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-light);
}

.logo {
  height: 64px;
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: 0 var(--space-lg);
  color: var(--primary);
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}

.logo-text {
  font-size: 18px;
  font-weight: 600;
}

.menu-wrapper {
  flex: 1;
  overflow-y: auto;
}

:deep(.n-menu) {
  padding: var(--space-md) 0;
}

:deep(.n-menu-item) {
  height: 48px;
  margin: var(--space-xs) var(--space-md);
  border-radius: var(--radius-md);
}

:deep(.n-menu-item-content) {
  padding: 0 var(--space-md) !important;
}

:deep(.n-menu-item-content__icon) {
  color: inherit !important;
  margin-right: var(--space-sm) !important;
}

/* 用户信息区域 */
.user-section {
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--border-light);
  flex-shrink: 0;
}

.user-info {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.user-details {
  flex: 1;
  min-width: 0;
}

.username {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-role {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
