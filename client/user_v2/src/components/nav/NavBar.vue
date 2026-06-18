<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useWebSocketStore } from '@/stores/websocket'
import { useResponsive } from '@/composables/useResponsive'
import { useThemeStore } from '@/stores/theme'
import { useNotificationStore } from '@/stores/notification'
import { useGlobalModal } from '@/composables/useGlobalModal'
import NotificationContent from '@/components/notification/NotificationContent.vue'
import PersonalizationModal from '@/components/quick/PersonalizationModal.vue'
import CreateRoomModal from '@/components/quick/CreateRoomModal.vue'
import { QuickBar, QuickDial } from '@/components/quick'
import type { QuickItem, QuickGroup } from '@/components/quick'
import { supportedLocales, setLocale, getCurrentLocale } from '@/i18n'
import {
  ChatRound,
  User,
  SwitchButton,
  Compass,
  Moon,
  Sunny,
  Bell,
  Connection,
  Setting,
  CirclePlus,
  Brush,
  Fold,
  Expand,
  MoreFilled,
} from '@element-plus/icons-vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const authStore = useAuthStore()
const settingsStore = useSettingsStore()
const wsStore = useWebSocketStore()
const themeStore = useThemeStore()
const notificationStore = useNotificationStore()
const globalModal = useGlobalModal()
const { isMobile, sidebarCollapsed, toggleSidebar } = useResponsive()

// 禁用自动属性继承，避免多根节点警告
// class 将显式绑定到 <nav> 元素
defineOptions({ inheritAttrs: false })

/**
 * 导航项配置
 */
const navItems = [
  { name: 'chat', path: '/app', icon: ChatRound, labelKey: 'chat.rooms' },
  { name: 'friends', path: '/friends', icon: Connection, labelKey: 'social.title' },
  { name: 'discover', path: '/discover', icon: Compass, labelKey: 'discover.title' },
  { name: 'profile', path: '/profile', icon: User, labelKey: 'profile.title' },
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
  if (item.name === 'discover') {
    return route.path === '/discover' || route.path.startsWith('/discover/')
  }
  return route.path === item.path
}

/**
 * 导航到指定路径
 * @param path - 目标路径
 */
async function navigate(path: string): Promise<void> {
  try {
    await router.push(path)
  } catch (error) {
    console.error('导航失败:', error)
  }
}

/**
 * 处理登出
 */
async function handleLogout(): Promise<void> {
  await authStore.logout()
  router.push('/login')
}

const currentLocale = computed(() => {
  const code = getCurrentLocale()
  return supportedLocales.find(l => l.code === code) ?? supportedLocales[1]!
})

function cycleLocale() {
  const codes = supportedLocales.map(l => l.code)
  const idx = codes.indexOf(currentLocale.value.code)
  const next = codes[(idx + 1) % codes.length] as 'en' | 'zh' | 'ja'
  setLocale(next)
  const serverLang = next === 'zh' ? 'zh-CN' : next === 'ja' ? 'ja-JP' : 'en-US'
  settingsStore.updateLocaleSettings({ language: serverLang })
}

/**
 * 打开创建房间弹窗
 */
function openCreateRoomModal() {
  globalModal.open({
    title: t('room.create'),
    component: CreateRoomModal,
    preset: 'card',
    closable: true,
    componentProps: {
      onCreated: (roomId: string) => {
        globalModal.close()
        if (wsStore.isConnected) {
          wsStore.send('JoinRoom', { room_id: roomId })
        }
        router.push('/app')
      },
      onCancel: () => {
        globalModal.close()
      },
    },
  })
}

function openPersonalization() {
  globalModal.open({
    title: t('quick.personalization'),
    component: PersonalizationModal,
    preset: 'card',
    closable: true,
  })
}

/**
 * QuickBar 配置 - 桌面端快捷操作
 * 外显：侧边栏切换、通知、主题
 * More 菜单：创建房间、个性化、语言、登出
 */
const quickItems = computed<QuickItem[]>(() => [
  {
    key: 'toggleSidebar',
    display: 'visible',
    type: 'action',
    icon: sidebarCollapsed.value ? Expand : Fold,
    label: sidebarCollapsed.value ? t('quick.sidebarExpand') : t('quick.sidebarCollapse'),
    onClick: toggleSidebar,
  },
  {
    key: 'notifications',
    display: 'visible',
    type: 'action',
    icon: Bell,
    label: t('quick.notifications'),
    badge: notificationStore.unreadCount,
    onClick: () => {
      if (globalModal.modalState.value.visible) {
        globalModal.close()
      } else {
        globalModal.open({
          title: t('quick.notifications'),
          component: NotificationContent,
          preset: 'card',
          closable: true,
        })
      }
    },
  },
  {
    key: 'theme',
    display: 'visible',
    type: 'action',
    icon: themeStore.isDark ? Moon : Sunny,
    label: themeStore.isDark ? t('quick.themeDark') : t('quick.themeLight'),
    onClick: () => {
      themeStore.toggleLightDark()
    },
  },
  {
    key: 'more',
    display: 'visible',
    type: 'menu',
    icon: MoreFilled,
    label: t('quick.more'),
    children: [
      { key: 'createRoom', icon: CirclePlus, label: t('room.create') },
      { key: 'personalization', icon: Brush, label: t('quick.personalization') },
      { key: 'locale', icon: currentLocale.value.flag, label: currentLocale.value.name },
      { key: 'logout', icon: SwitchButton, label: t('common.logout') },
    ],
    onSelect: (childKey: string) => {
      switch (childKey) {
        case 'createRoom':
          openCreateRoomModal()
          break
        case 'personalization':
          openPersonalization()
          break
        case 'locale':
          cycleLocale()
          break
        case 'logout':
          handleLogout()
          break
      }
    },
  },
])

/**
 * QuickDial 分组配置 - 仅移动端使用
 * 全部以独立按钮展示，无子菜单
 */
const quickGroups = computed<QuickGroup[]>(() => [
  {
    key: 'main',
    icon: Compass,
    label: t('quick.groupMain'),
    items: [
      {
        key: 'toggleSidebar',
        display: 'visible',
        type: 'action',
        icon: sidebarCollapsed.value ? Expand : Fold,
        label: sidebarCollapsed.value ? t('quick.sidebarExpand') : t('quick.sidebarCollapse'),
        onClick: toggleSidebar,
      },
      {
        key: 'createRoom',
        display: 'visible',
        type: 'action',
        icon: CirclePlus,
        label: t('room.create'),
        onClick: openCreateRoomModal,
      },
      {
        key: 'notifications',
        display: 'visible',
        type: 'action',
        icon: Bell,
        label: t('quick.notifications'),
        badge: notificationStore.unreadCount,
        onClick: () => {
          if (globalModal.modalState.value.visible) {
            globalModal.close()
          } else {
            globalModal.open({
              title: t('quick.notifications'),
              component: NotificationContent,
              preset: 'card',
              closable: true,
            })
          }
        },
      },
      {
        key: 'personalization',
        display: 'visible',
        type: 'action',
        icon: Brush,
        label: t('quick.personalization'),
        onClick: openPersonalization,
      },
      {
        key: 'theme',
        display: 'visible',
        type: 'action',
        icon: themeStore.isDark ? Moon : Sunny,
        label: themeStore.isDark ? t('quick.themeDark') : t('quick.themeLight'),
        onClick: () => {
          themeStore.toggleLightDark()
        },
      },
    ],
  },
  {
    key: 'system',
    icon: Setting,
    label: t('quick.groupSystem'),
    items: [
      {
        key: 'locale',
        display: 'visible',
        type: 'action',
        icon: currentLocale.value.flag,
        label: currentLocale.value.name,
        onClick: cycleLocale,
      },
      {
        key: 'logout',
        display: 'visible',
        type: 'action',
        icon: SwitchButton,
        label: t('common.logout'),
        onClick: handleLogout,
      },
    ],
  },
])
</script>

<template>
  <nav class="nav-bar" :class="{ 'nav-bar--mobile': isMobile }" v-bind="$attrs">
    <!-- 桌面端：Logo区域 -->
    <div v-if="!isMobile" class="nav-bar__logo">
      <div class="nav-bar__logo-wrapper">
        <img src="/favicon.svg" alt="CapellaRoom" class="logo-img" />
      </div>
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

    <!-- 底部区域：QuickBar -->
    <div v-if="!isMobile" class="nav-bar__footer">
      <QuickBar :items="quickItems" />
    </div>

  </nav>

  <!-- 移动端 FAB 快捷拨号 -->
  <QuickDial v-if="isMobile" :groups="quickGroups" />
</template>

<style scoped lang="scss">
.nav-bar {
  width: 56px;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border);
  flex-shrink: 0;
}

/* Logo区域 */
.nav-bar__logo {
  width: 100%;
  padding: 16px 0;
  display: flex;
  justify-content: center;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.nav-bar__logo-wrapper {
  position: relative;
  display: inline-flex;

  .logo-img {
    width: 32px;
    height: 32px;
    filter: var(--logo-filter);
  }
}

/* Nav items */
.nav-bar__items {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 0;
  overflow-y: auto;
}

.nav-bar__item {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 8px;
  color: var(--muted);
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;

  &:hover {
    color: var(--accent);
    background: var(--accent-soft);
  }

  &--active {
    color: var(--accent);
    background: var(--accent-soft);

    &::before {
      content: '';
      position: absolute;
      left: -8px;
      top: 50%;
      transform: translateY(-50%);
      width: 3px;
      height: 20px;
      background: var(--accent);
      border-radius: 0 3px 3px 0;
    }
  }
}

/* 底部区域 */
.nav-bar__footer {
  width: 100%;
  padding: 8px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}



/* 移动端样式 */
.nav-bar--mobile {
  width: 100%;
  height: 56px;
  flex-direction: row;
  border-right: none;
  border-top: 1px solid var(--border);
  position: fixed;
  bottom: 0;
  left: 0;
  z-index: 200;

  .nav-bar__items {
    flex-direction: row;
    justify-content: center;
    padding: 0;
    gap: 0;
    flex: 1;
  }

  .nav-bar__item {
    flex: 1;
    max-width: 72px;
    height: 100%;
    border-radius: 0;

    &:hover,
    &--active {
      background: transparent;
      color: var(--accent);
    }

    &--active::before {
      display: none;
    }

    &--active::after {
      display: none;
    }
  }

  .nav-bar__logout {
    width: auto;
    padding: 0 8px;
    border-left: 1px solid var(--border);
  }

  .nav-bar__footer {
    display: none;
  }
}
</style>
