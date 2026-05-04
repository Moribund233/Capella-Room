<template>
  <header class="app-header" :class="{ 'is-scrolled': isScrolled }">
    <div class="header-left">
      <!-- Logo/品牌名 -->
      <div class="brand">
        <div v-if="appConfig.logo" class="logo-wrapper">
          <img :src="appConfig.logo" class="logo-img" :alt="appConfig.name" />
        </div>
        <div v-else class="logo">
          <span class="logo-text">{{ appNameFirstChar }}</span>
        </div>
        <span v-if="showBrandText" class="brand-text">{{ appConfig.name }}</span>
      </div>

      <!-- 面包屑/页面标题区域 -->
      <div class="header-divider"></div>
      <div class="page-info">
        <span class="page-title">{{ currentPageTitle }}</span>
      </div>
    </div>

    <div class="header-right">
      <!-- QuickBar 快捷工具栏 -->
      <div class="quickbar-wrapper">
        <QuickBar :items="quickBarItems" />
      </div>

      <!-- 分隔线 -->
      <div class="header-divider vertical"></div>

      <!-- 用户头像 -->
      <div class="user-profile" @click="handleUserClick">
        <div class="avatar-wrapper">
          <img v-if="userAvatar" :src="userAvatar" class="avatar-img" :alt="username" />
          <div v-else class="avatar">
            <span v-if="usernameFirstChar" class="avatar-text">{{ usernameFirstChar }}</span>
            <User v-else class="avatar-icon" :size="18" />
          </div>
          <div class="avatar-status" :class="onlineStatusClass"></div>
        </div>
        <div v-if="layoutStore.isDesktop" class="user-info">
          <span class="username">{{ username }}</span>
          <span class="user-role">用户</span>
        </div>
        <ChevronDown v-if="layoutStore.isDesktop" class="dropdown-icon" :size="14" />
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { User, ChevronDown } from 'lucide-vue-next'
import { useAppConfig, useQuickBarConfig } from '@/composables'
import { useLayoutStore, useAuthStore, useWebSocketStore } from '@/store'
import QuickBar from '@/components/common/QuickBar.vue'

/**
 * AppHeader 组件
 * 现代化玻璃态页面头部导航
 */

const route = useRoute()
const appConfig = useAppConfig()
const { quickBarItems } = useQuickBarConfig()
const layoutStore = useLayoutStore()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()

/** 滚动状态 */
const isScrolled = ref(false)

/** 监听滚动 */
const handleScroll = () => {
  isScrolled.value = window.scrollY > 10
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll, { passive: true })
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll)
})

/** 应用名称首字符（用于默认 Logo） */
const appNameFirstChar = computed(() => {
  return appConfig.name.charAt(0).toUpperCase()
})

/** 是否显示品牌文字 */
const showBrandText = computed(() => {
  return layoutStore.isDesktop || (layoutStore.isTablet && !layoutStore.isSidebarCollapsed)
})

/** 当前页面标题 */
const currentPageTitle = computed(() => {
  return (route.meta?.title as string) || '首页'
})

/** 当前用户名 */
const username = computed(() => {
  return authStore.username || '访客'
})

/** 用户头像 */
const userAvatar = computed(() => {
  return authStore.user?.avatar_url || null
})

/** 用户名字首字符（用于默认头像） */
const usernameFirstChar = computed(() => {
  return username.value.charAt(0).toUpperCase()
})

/** 在线状态样式类 */
const onlineStatusClass = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return 'status-online'
    case 'connecting':
    case 'reconnecting':
      return 'status-busy'
    default:
      return 'status-offline'
  }
})

/** 处理用户点击 */
const handleUserClick = () => {
  // 可以触发用户菜单展开
}
</script>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header-height);
  width: var(--header-compact-width, 100vw);
  margin: 0 var(--header-margin, 0);
  padding: 0 var(--space-5);
  background: var(--header-bg);
  backdrop-filter: var(--glass-backdrop);
  -webkit-backdrop-filter: var(--glass-backdrop);
  box-shadow: var(--header-shadow);
  position: fixed;
  top: 0;
  left: 0;
  z-index: 100;
  transition: all var(--duration-slow) var(--ease-out-expo);
  border-radius: 0 0 var(--header-border-radius) var(--header-border-radius);
  opacity: var(--header-opacity);
  border: var(--layout-border-width) var(--layout-border-style) var(--layout-border-color);
  border-top: none;
}

/* 滚动后的效果 */
.app-header.is-scrolled {
  background: rgba(255, 255, 255, 0.95);
  box-shadow: var(--shadow-md);
}

/* 左侧区域 */
.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.logo-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--bg-sunken);
}

.logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--color-primary-gradient);
  color: white;
  font-weight: 700;
  font-size: 16px;
  border-radius: var(--radius-lg);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
  transition: transform var(--duration-normal) var(--ease-spring);
}

.logo:hover {
  transform: scale(1.05);
}

.logo-text {
  background: linear-gradient(135deg, #ffffff 0%, rgba(255, 255, 255, 0.9) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.brand-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

/* 分隔线 */
.header-divider {
  width: 1px;
  height: 24px;
  background: var(--border-color-base);
}

.header-divider.vertical {
  width: 1px;
  height: 24px;
}

/* 页面信息 */
.page-info {
  display: flex;
  align-items: center;
}

.page-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

/* 右侧区域 */
.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.quickbar-wrapper {
  display: flex;
  align-items: center;
}

/* 用户资料 */
.user-profile {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-smooth);
}

.user-profile:hover {
  background: var(--bg-hover);
}

.avatar-wrapper {
  position: relative;
  width: 36px;
  height: 36px;
}

.avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--bg-sunken);
  border-radius: 50%;
  border: 2px solid var(--border-color-base);
  transition: all var(--duration-fast) var(--ease-smooth);
}

.avatar-img {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid var(--border-color-base);
}

.avatar-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-primary);
}

.avatar-icon {
  color: var(--text-tertiary);
}

.avatar-status {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid var(--header-bg);
  transition: all var(--duration-fast) var(--ease-smooth);
}

.status-online {
  background-color: var(--color-success);
}

.status-busy {
  background-color: var(--color-warning);
}

.status-offline {
  background-color: var(--color-error);
}

.user-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.username {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
}

.user-role {
  font-size: 12px;
  color: var(--text-tertiary);
}

.dropdown-icon {
  color: var(--text-tertiary);
  transition: transform var(--duration-fast) var(--ease-smooth);
}

.user-profile:hover .dropdown-icon {
  transform: rotate(180deg);
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .app-header {
    height: var(--header-height-mobile);
    padding: 0 var(--space-4);
  }

  .header-divider {
    display: none;
  }

  .page-info {
    display: none;
  }
}
</style>
