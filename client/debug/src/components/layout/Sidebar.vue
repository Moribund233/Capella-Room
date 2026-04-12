<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  LayoutDashboard,
  Wifi,
  Plug,
  MessageSquare,
  Users,
  TestTube,
  LogOut,
  CheckCircle,
  Menu,
  X
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

// 桌面端侧边栏折叠状态
const isCollapsed = ref(false)
// 移动端抽屉开关状态
const mobileDrawerOpen = ref(false)

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

// 切换桌面端侧边栏
const toggleDesktopSidebar = () => {
  isCollapsed.value = !isCollapsed.value
}

// 切换移动端抽屉
const toggleMobileDrawer = () => {
  mobileDrawerOpen.value = !mobileDrawerOpen.value
}

const handleMenuClick = (key: string) => {
  const item = menuItems.find((item) => item.key === key)
  if (item) {
    router.push(item.path)
  }
}

const handleLogout = async () => {
  try {
    await authStore.logout()
    router.replace('/login')
  } catch (err) {
    console.error('登出失败:', err)
    router.replace('/login')
  }
}
</script>

<template>
  <div>
    <!-- 桌面端侧边栏 -->
    <div
      class="desktop-sidebar"
      :class="{ 'sidebar-collapsed': isCollapsed }"
    >
      <!-- Logo区域 -->
      <div class="sidebar-header">
        <div class="logo">
          <TestTube class="icon-md" />
          <span v-if="!isCollapsed" class="logo-text">Seredeli</span>
        </div>
      </div>

      <!-- 菜单区域 -->
      <div class="menu-wrapper">
        <n-menu
          :value="activeKey"
          :collapsed="isCollapsed"
          :collapsed-width="64"
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

      <!-- 底部用户区域 -->
      <div class="sidebar-footer">
        <div v-if="!isCollapsed" class="user-info">
          <n-avatar size="small" :style="{ backgroundColor: 'var(--primary)' }">
            {{ authStore.userAvatar }}
          </n-avatar>
          <div class="user-details">
            <div class="username">{{ authStore.username }}</div>
            <div class="user-role">{{ authStore.roleText }}</div>
          </div>
        </div>
        <n-button
          v-else
          text
          type="error"
          size="small"
          @click="handleLogout"
          class="logout-btn-collapsed"
        >
          <template #icon>
            <LogOut class="icon-md" />
          </template>
        </n-button>
        <n-button
          v-if="!isCollapsed"
          text
          type="error"
          size="small"
          @click="handleLogout"
        >
          <template #icon>
            <LogOut class="icon-sm" />
          </template>
          退出
        </n-button>
      </div>
    </div>

    <!-- 移动端抽屉侧边栏 -->
    <div
      class="mobile-drawer"
      :class="{ 'drawer-open': mobileDrawerOpen }"
    >
      <div class="drawer-overlay" @click="mobileDrawerOpen = false"></div>
      <div class="drawer-content">
        <div class="drawer-header">
          <div class="logo">
            <TestTube class="icon-md" />
            <span class="logo-text">Seredeli</span>
          </div>
        </div>

        <div class="drawer-menu">
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

        <div class="drawer-footer">
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
    </div>

    <!-- 固定右上角菜单按钮 -->
    <div class="menu-toggle-btn" @click="toggleDesktopSidebar">
      <Menu v-if="!isCollapsed" class="icon-lg" />
      <X v-else class="icon-lg" />
    </div>

    <!-- 移动端菜单按钮 -->
    <div class="mobile-menu-btn" @click="toggleMobileDrawer">
      <Menu v-if="!mobileDrawerOpen" class="icon-lg" />
      <X v-else class="icon-lg" />
    </div>
  </div>
</template>

<script lang="ts">
import { h } from 'vue'
export { h }
</script>

<style scoped>
/* 桌面端侧边栏 */
.desktop-sidebar {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 240px;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  transition: width 0.3s ease;
  z-index: 100;
}

.desktop-sidebar.sidebar-collapsed {
  width: 64px;
}

/* 移动端隐藏桌面侧边栏 */
@media screen and (max-width: 767px) {
  .desktop-sidebar {
    display: none;
  }
}

/* 桌面端隐藏移动端元素 */
@media screen and (min-width: 768px) {
  .mobile-drawer,
  .mobile-menu-btn {
    display: none;
  }
}

/* 侧边栏头部 */
.sidebar-header {
  height: 64px;
  display: flex;
  align-items: center;
  padding: 0 var(--space-md);
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  color: var(--primary);
}

.logo-text {
  font-size: 18px;
  font-weight: 600;
  white-space: nowrap;
}

/* 菜单区域 */
.menu-wrapper {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm) 0;
}

:deep(.n-menu) {
  padding: var(--space-xs) 0;
}

:deep(.n-menu-item) {
  height: 48px;
  margin: var(--space-xs) var(--space-sm);
  border-radius: var(--radius-md);
}

:deep(.n-menu-item-content) {
  padding: 0 var(--space-md) !important;
}

:deep(.n-menu-item-content__icon) {
  color: inherit !important;
  margin-right: var(--space-sm) !important;
}

/* 侧边栏底部 */
.sidebar-footer {
  padding: var(--space-md);
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
  overflow: hidden;
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

.logout-btn-collapsed {
  width: 100%;
  display: flex;
  justify-content: center;
}

/* 固定右上角菜单按钮 - 桌面端 */
.menu-toggle-btn {
  position: fixed;
  top: var(--space-md);
  right: var(--space-md);
  width: 40px;
  height: 40px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 200;
  color: var(--text-primary);
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.menu-toggle-btn:hover {
  background-color: var(--bg-tertiary);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* 移动端隐藏桌面菜单按钮 */
@media screen and (max-width: 767px) {
  .menu-toggle-btn {
    display: none;
  }
}

/* 移动端菜单按钮 */
.mobile-menu-btn {
  position: fixed;
  top: var(--space-md);
  right: var(--space-md);
  width: 40px;
  height: 40px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 200;
  color: var(--text-primary);
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.mobile-menu-btn:hover {
  background-color: var(--bg-tertiary);
}

/* 桌面端隐藏移动端菜单按钮 */
@media screen and (min-width: 768px) {
  .mobile-menu-btn {
    display: none;
  }
}

/* 移动端抽屉 */
.mobile-drawer {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 150;
  visibility: hidden;
  opacity: 0;
  transition: visibility 0.3s, opacity 0.3s;
}

.mobile-drawer.drawer-open {
  visibility: visible;
  opacity: 1;
}

.drawer-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
}

.drawer-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 280px;
  max-width: 80vw;
  height: 100%;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  transform: translateX(-100%);
  transition: transform 0.3s ease;
}

.drawer-open .drawer-content {
  transform: translateX(0);
}

.drawer-header {
  height: 64px;
  display: flex;
  align-items: center;
  padding: 0 var(--space-md);
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
}

.drawer-menu {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm) 0;
}

.drawer-footer {
  padding: var(--space-md);
  border-top: 1px solid var(--border-light);
  flex-shrink: 0;
}
</style>
