<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useResponsive } from '@/composables/useResponsive'
import { MessageSquare, User, Settings } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const { isMobile } = useResponsive()

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
</script>

<template>
  <nav class="nav-bar" :class="{ 'nav-bar--mobile': isMobile }">
    <!-- Logo -->
    <div class="nav-bar__logo" @click="navigate('/')">
      <span class="nav-bar__logo-text">S</span>
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

    <!-- User avatar -->
    <div v-if="authStore.user" class="nav-bar__user">
      <button
        class="nav-bar__avatar"
        @click="navigate('/profile')"
        :title="authStore.user.username"
      >
        {{ authStore.user.username.charAt(0).toUpperCase() }}
      </button>
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

/* Logo */
.nav-bar__logo {
  width: 100%;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--color-divider);
  cursor: pointer;
  flex-shrink: 0;
}

.nav-bar--mobile .nav-bar__logo {
  display: none;
}

.nav-bar__logo-text {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-primary);
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

/* User area */
.nav-bar__user {
  padding: 8px 0;
  border-top: 1px solid var(--color-divider);
  width: 100%;
  display: flex;
  justify-content: center;
}

.nav-bar--mobile .nav-bar__user {
  display: none;
}

.nav-bar__avatar {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-full);
  background: var(--color-primary);
  color: var(--color-white);
  border: none;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity var(--duration-fast);
}

.nav-bar__avatar:hover {
  opacity: 0.85;
}
</style>
