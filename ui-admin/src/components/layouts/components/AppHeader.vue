<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useThemeStore } from '@/stores/theme'
import { useUserStore } from '@/stores/user'
import {
  Menu,
  Bell,
  FullScreen,
  Moon,
  Sunny,
  ArrowDown,
  User,
  Setting,
  SwitchButton,
} from '@element-plus/icons-vue'

/**
 * 顶部导航栏组件
 * 提供搜索、通知、主题切换、用户菜单等功能
 */

/** 定义props */
const props = defineProps<{
  collapsed?: boolean
}>()

/** 定义事件 */
const emit = defineEmits<{
  'toggle-sidebar': []
}>()

/** 路由 */
const router = useRouter()

/** 用户状态 */
const userStore = useUserStore()

/** 侧边栏宽度 */
const sidebarWidth = computed(() => (props.collapsed ? '64px' : '240px'))

/** 主题状态 */
const themeStore = useThemeStore()

/** 用户下拉菜单显示状态 */
const showUserDropdown = ref(false)

/** 通知数量 */
const notificationCount = ref(3)

/** 用户显示名称 */
const displayName = computed(() => userStore.displayName || 'Admin')

/** 用户头像 */
const avatarText = computed(() => {
  const name = userStore.displayName
  return name ? name.charAt(0).toUpperCase() : 'A'
})

/**
 * 切换主题
 */
const toggleTheme = () => {
  themeStore.toggleTheme()
}

/**
 * 切换全屏
 */
const toggleFullscreen = () => {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen()
  } else {
    document.exitFullscreen()
  }
}

/**
 * 切换侧边栏
 */
const toggleSidebar = () => {
  emit('toggle-sidebar')
}

/**
 * 处理退出登录
 * 调用 UserStore 的 logout 方法，清除登录状态并跳转到登录页
 */
const handleLogout = async () => {
  // 关闭下拉菜单
  showUserDropdown.value = false

  // 执行登出
  await userStore.logout()

  // 跳转到登录页
  router.push('/login')
}
</script>

<template>
  <header class="header">
    <!-- 左侧：面包屑/标题 -->
    <div class="header-left">
      <h1 class="page-title">管理后台</h1>
    </div>

    <!-- 右侧：工具栏 -->
    <div class="header-right">
      <!-- 主题切换 -->
      <button class="toolbar-btn" @click="toggleTheme">
        <Moon v-if="themeStore.isDark" />
        <Sunny v-else />
      </button>

      <!-- 全屏 -->
      <button class="toolbar-btn" @click="toggleFullscreen">
        <FullScreen />
      </button>

      <!-- 通知 -->
      <button class="toolbar-btn notification-btn">
        <Bell />
        <span v-if="notificationCount > 0" class="notification-badge">{{ notificationCount }}</span>
      </button>

      <!-- 侧边栏折叠按钮 -->
      <button class="toolbar-btn sidebar-toggle-btn" @click="toggleSidebar">
        <Menu />
      </button>

      <!-- 用户菜单 -->
      <div class="user-menu" :class="{ 'is-open': showUserDropdown }">
        <button class="user-trigger" @click="showUserDropdown = !showUserDropdown">
          <div class="avatar">{{ avatarText }}</div>
          <span class="username">{{ displayName }}</span>
          <ArrowDown class="arrow-icon" />
        </button>

        <div class="dropdown-menu">
          <div class="dropdown-item">
            <User />
            <span>个人中心</span>
          </div>
          <div class="dropdown-item">
            <Setting />
            <span>账号设置</span>
          </div>
          <div class="dropdown-divider"></div>
          <div class="dropdown-item" @click="handleLogout">
            <SwitchButton />
            <span>退出登录</span>
          </div>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.header {
  position: fixed;
  top: 0;
  right: 0;
  left: v-bind(sidebarWidth);
  height: var(--header-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-6);
  background-color: var(--header-bg);
  border-bottom: 1px solid var(--header-border);
  transition: left var(--transition-normal);
  z-index: calc(var(--z-fixed) - 1);
}

/* 左侧 */
.header-left {
  display: flex;
  align-items: center;
}

.page-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--header-text);
}

/* 右侧工具栏 */
.header-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
}

.toolbar-btn {
  position: relative;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  background-color: transparent;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.toolbar-btn svg {
  width: 18px;
  height: 18px;
}

.toolbar-btn:hover {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

.notification-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: var(--font-weight-bold);
  color: white;
  background-color: var(--error);
  border-radius: var(--radius-full);
}

/* 用户菜单 */
.user-menu {
  position: relative;
  margin-left: var(--spacing-2);
}

.user-trigger {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-1) var(--spacing-2) var(--spacing-1) var(--spacing-1);
  background-color: transparent;
  border-radius: var(--radius-md);
  transition: background-color var(--transition-fast);
}

.user-trigger:hover {
  background-color: var(--bg-secondary);
}

.avatar {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--primary-alpha);
  color: var(--primary);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  border-radius: var(--radius-full);
}

.username {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-primary);
}

.arrow-icon {
  width: 16px;
  height: 16px;
  color: var(--text-tertiary);
  transition: transform var(--transition-fast);
}

.user-menu.is-open .arrow-icon {
  transform: rotate(180deg);
}

/* 下拉菜单 */
.dropdown-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 180px;
  padding: var(--spacing-2);
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  opacity: 0;
  visibility: hidden;
  transform: translateY(-8px);
  transition: all var(--transition-fast);
}

.user-menu.is-open .dropdown-menu {
  opacity: 1;
  visibility: visible;
  transform: translateY(0);
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-2) var(--spacing-3);
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.dropdown-item svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.dropdown-item:hover {
  background-color: var(--bg-secondary);
}

.dropdown-divider {
  height: 1px;
  margin: var(--spacing-2) 0;
  background-color: var(--border-secondary);
}

/* 响应式 */
@media (max-width: 1023px) {
  .header {
    left: 0 !important;
  }

  .username {
    display: none;
  }
}

@media (max-width: 767px) {
  .header {
    padding: 0 var(--spacing-4);
  }

  .header-center {
    display: none;
  }

  .toolbar-btn {
    width: 36px;
    height: 36px;
  }

  .header-right {
    gap: var(--spacing-1);
  }

  .user-menu {
    margin-left: var(--spacing-1);
  }

  .user-trigger {
    padding: var(--spacing-1);
  }
}
</style>
