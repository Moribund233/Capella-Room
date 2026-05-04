<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useResponsive } from '@/composables/useResponsive'
import { MessageSquare, User, Settings, LogOut } from 'lucide-vue-next'
import { QuickBar } from '@/components/quick'
import { useQuickBar } from '@/composables/quick'
import { quickBarConfig } from '@/config/quick'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const { isMobile } = useResponsive()

// 初始化 QuickBar
const { items: quickItems } = useQuickBar(quickBarConfig)

const navItems = [
  { name: 'chat', path: '/', icon: MessageSquare, label: '聊天' },
  { name: 'profile', path: '/profile', icon: User, label: '个人' },
  { name: 'settings', path: '/settings', icon: Settings, label: '设置' },
]

function isActive(item: (typeof navItems)[0]): boolean {
  if (item.name === 'chat') {
    return ['chat', 'chat-room'].includes(route.name as string)
  }
  return route.name === item.name
}

function navigate(path: string) {
  router.push(path)
}

/**
 * 处理登出
 */
async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}
</script>

<template>
  <nav class="nav-bar" :class="{ 'nav-bar--mobile': isMobile }">
    <!-- 登出按钮 -->
    <div class="nav-bar__logout">
      <button
        class="nav-bar__logout-btn"
        @click="handleLogout"
        title="登出"
        aria-label="登出"
      >
        <LogOut :size="20" />
      </button>
    </div>

    <!-- Nav items -->
    <div class="nav-bar__items">
      <button
        v-for="item in navItems"
        :key="item.name"
        class="nav-bar__item"
        :class="{ 'nav-bar__item--active': isActive(item) }"
        @click="navigate(item.path)"
        :title="item.label"
      >
        <component :is="item.icon" :size="22" />
      </button>
    </div>

    <!-- QuickBar 快捷栏（仅桌面端） -->
    <div v-if="!isMobile" class="nav-bar__quick">
      <QuickBar :items="quickItems" position="header" />
    </div>
  </nav>
</template>

<style scoped>
.nav-bar {
  width: 56px;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: var(--color-white);
  border-right: 1px solid var(--color-border);
  flex-shrink: 0;
  overflow: hidden;
}

.nav-bar--mobile {
  width: 100%;
  height: 56px;
  flex-direction: row;
  border-right: none;
  border-top: 1px solid var(--color-border);
  position: fixed;
  bottom: 0;
  left: 0;
  z-index: 200;
}

/* 登出按钮 */
.nav-bar__logout {
  width: 100%;
  padding: 8px 0;
  display: flex;
  justify-content: center;
  border-bottom: 1px solid var(--color-divider);
  flex-shrink: 0;
}

.nav-bar--mobile .nav-bar__logout {
  width: auto;
  padding: 0 8px;
  border-bottom: none;
  border-right: 1px solid var(--color-divider);
}

.nav-bar__logout-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-default);
}

.nav-bar__logout-btn:hover {
  color: var(--color-danger);
  background: var(--color-danger-light, rgba(239, 68, 68, 0.1));
}

/* Nav items */
.nav-bar__items {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 0;
}

.nav-bar--mobile .nav-bar__items {
  flex-direction: row;
  justify-content: center;
  padding: 0;
  gap: 0;
}

.nav-bar__item {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-default);
  position: relative;
}

.nav-bar--mobile .nav-bar__item {
  flex: 1;
  max-width: 72px;
  height: 100%;
  border-radius: 0;
}

.nav-bar__item:hover {
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.nav-bar__item--active {
  color: var(--color-primary);
  background: var(--color-primary-light);
}

/* Active indicator pill */
.nav-bar__item--active::before {
  content: '';
  position: absolute;
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  border-radius: 0 3px 3px 0;
  background: var(--color-primary);
}

.nav-bar--mobile .nav-bar__item--active::before {
  left: 50%;
  top: 0;
  transform: translateX(-50%);
  width: 20px;
  height: 3px;
  border-radius: 0 0 3px 3px;
}

/* QuickBar 区域 */
.nav-bar__quick {
  padding: 8px 0;
  border-top: 1px solid var(--color-divider);
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.nav-bar--mobile .nav-bar__quick {
  display: none;
}
</style>
