<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  ChatRound,
  User,
  Setting,
  SwitchButton,
} from '@element-plus/icons-vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

/**
 * 导航项配置
 */
const navItems = [
  { name: 'chat', path: '/app', icon: ChatRound, labelKey: 'chat.channels' },
  { name: 'profile', path: '/profile', icon: User, labelKey: 'profile.title' },
  { name: 'settings', path: '/settings', icon: Setting, labelKey: 'profile.preferences.title' },
]

/**
 * 判断导航项是否激活
 * @param item - 导航项
 * @returns 是否激活
 */
function isActive(item: (typeof navItems)[0]): boolean {
  if (item.name === 'chat') {
    return route.path === '/app' || route.path.startsWith('/app/')
  }
  return route.path === item.path
}

/**
 * 导航到指定路径
 * @param path - 目标路径
 */
function navigate(path: string): void {
  router.push(path)
}

/**
 * 处理登出
 */
function handleLogout(): void {
  // TODO: 实现登出逻辑
  router.push('/login')
}
</script>

<template>
  <nav class="nav-bar">
    <!-- Logo区域 -->
    <div class="nav-bar__logo">
      <div class="logo-mark">W</div>
    </div>

    <!-- 导航项 -->
    <div class="nav-bar__items">
      <button
        v-for="item in navItems"
        :key="item.name"
        class="nav-bar__item"
        :class="{ 'nav-bar__item--active': isActive(item) }"
        @click="navigate(item.path)"
        :title="t(item.labelKey)"
      >
        <el-icon :size="22">
          <component :is="item.icon" />
        </el-icon>
      </button>
    </div>

    <!-- 底部操作区 -->
    <div class="nav-bar__footer">
      <slot name="quick-bar" />
      
      <!-- 登出按钮 -->
      <button
        class="nav-bar__item nav-bar__item--logout"
        @click="handleLogout"
        :title="t('common.logout')"
      >
        <el-icon :size="20">
          <SwitchButton />
        </el-icon>
      </button>
    </div>
  </nav>
</template>

<style scoped lang="scss">
.nav-bar {
  width: 64px;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: var(--wave-surface);
  border-right: 1px solid var(--wave-border);
  flex-shrink: 0;
  padding: 12px 0;
}

.nav-bar__logo {
  padding: 8px 0 16px;
  margin-bottom: 8px;
  border-bottom: 1px solid var(--wave-border);
  width: 100%;
  display: flex;
  justify-content: center;
}

.logo-mark {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: linear-gradient(135deg, var(--wave-accent), var(--wave-accent-pink));
  display: grid;
  place-items: center;
  font-size: 20px;
  font-weight: 700;
  color: #fff;
  font-family: var(--wave-font-display);
}

.nav-bar__items {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  padding: 0 8px;
}

.nav-bar__item {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--wave-muted);
  cursor: pointer;
  transition: all 0.2s ease;
  margin: 0 auto;

  &:hover {
    background: var(--wave-message-hover);
    color: var(--wave-fg);
  }

  &--active {
    background: var(--wave-accent);
    color: #fff;

    &:hover {
      background: var(--wave-accent);
      color: #fff;
    }
  }

  &--logout {
    margin-top: 8px;
    
    &:hover {
      color: var(--el-color-danger);
      background: var(--el-color-danger-light-9);
    }
  }
}

.nav-bar__footer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 100%;
  padding: 8px;
  border-top: 1px solid var(--wave-border);
  margin-top: auto;
}
</style>
