<template>
  <header class="app-header">
    <div class="header-left">
      <!-- Logo/品牌名 -->
      <div class="brand">
        <img v-if="appConfig.logo" :src="appConfig.logo" class="logo-img" :alt="appConfig.name" />
        <div v-else class="logo">{{ appNameFirstChar }}</div>
        <span v-if="showBrandText" class="brand-text">{{ appConfig.name }}</span>
      </div>
    </div>

    <div class="header-right">
      <!-- QuickBar 快捷工具栏 -->
      <QuickBar :items="quickBarItems" />

      <!-- 用户头像 -->
      <div class="user-profile">
        <div class="avatar">
          <User class="avatar-icon" :size="18" />
        </div>
        <span v-if="layoutStore.isDesktop" class="username">{{ username }}</span>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { User } from 'lucide-vue-next'
import { useAppConfig, useQuickBarConfig } from '@/composables'
import { useLayoutStore, useAuthStore } from '@/store'
import QuickBar from '@/components/common/QuickBar.vue'

/**
 * AppHeader 组件
 * 页面头部导航，包含 Logo、QuickBar、用户信息
 */

/** 应用配置 */
const appConfig = useAppConfig()

/** QuickBar 配置 */
const { quickBarItems } = useQuickBarConfig()

/** Layout Store */
const layoutStore = useLayoutStore()

/** Auth Store */
const authStore = useAuthStore()

/** 应用名称首字符（用于默认 Logo） */
const appNameFirstChar = computed(() => {
  return appConfig.name.charAt(0).toUpperCase()
})

/** 是否显示品牌文字（桌面端或平板端展开时） */
const showBrandText = computed(() => {
  return layoutStore.isDesktop || (layoutStore.isTablet && !layoutStore.isSidebarCollapsed)
})

/** 当前用户名 */
const username = computed(() => {
  return authStore.username || '访客'
})
</script>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header-height);
  width: var(--header-compact-width, 100vw);
  margin: 0 var(--header-margin, 0);
  padding: 0 16px;
  background: var(--header-bg);
  box-shadow: var(--header-shadow);
  position: fixed;
  top: 0;
  left: 0;
  z-index: 100;
  transition: var(--transition-base);
  border-radius: 0 0 var(--header-border-radius) var(--header-border-radius);
  opacity: var(--header-opacity);
  border: var(--layout-border-width) var(--layout-border-style) var(--layout-border-color);
  border-top: none;
}

/* 左侧区域 */
.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-hover) 100%);
  color: white;
  font-weight: 700;
  font-size: 16px;
  border-radius: 8px;
}

.logo-img {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  object-fit: contain;
  filter: var(--logo-filter, none);
  transition: filter var(--transition-base);
}

.brand-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

/* 右侧区域 */
.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-profile {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 8px;
}

.avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--color-primary-light);
  border-radius: 50%;
}

.avatar-icon {
  color: var(--color-primary);
}

.username {
  font-size: 14px;
  color: var(--text-primary);
  white-space: nowrap;
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .app-header {
    height: var(--header-height-mobile);
    padding: 0 12px;
  }
}
</style>
