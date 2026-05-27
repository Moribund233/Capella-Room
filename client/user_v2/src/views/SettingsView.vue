<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NavBar } from '@/components/nav'
import { QuickBar } from '@/components/quick'
import type { QuickItem } from '@/components/quick'
import { useTheme } from '@/composables/useTheme'
import {
  ArrowLeft,
  Bell,
  Brush,
  View,
  ChatLineSquare,
  Moon,
  Sunny,
  Bell as BellIcon,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()
const { isDark, toggleTheme } = useTheme()

// QuickBar 配置 - 主题图标根据当前主题动态变化
const quickItems = computed<QuickItem[]>(() => [
  {
    key: 'notifications',
    display: 'visible',
    icon: BellIcon,
    label: t('profile.preferences.notifications.title'),
    badge: 3,
    onClick: () => {},
  },
  {
    key: 'theme',
    display: 'visible',
    icon: isDark.value ? Moon : Sunny,
    label: isDark.value ? t('profile.appearance.theme.dark') : t('profile.appearance.theme.light'),
    onClick: toggleTheme,
  },
])

// 当前激活的设置标签
const activeTab = ref('notifications')

// 通知设置
const notificationSettings = ref({
  enabled: true,
  mentions: true,
  replies: true,
  directMessages: true,
  roomInvites: true,
  soundEffects: false,
  messagePreview: true,
})

// 隐私设置
const privacySettings = ref({
  showOnlineStatus: true,
  allowFriendRequests: true,
  allowRoomInvites: true,
  showActivity: true,
})

// 消息设置
const messageSettings = ref({
  enterToSend: true,
  showTimestamps: true,
  compactMode: false,
  autoLoadImages: true,
})

// 主题设置
const themeSettings = ref({
  theme: 'dark',
  messageDensity: 'comfortable',
  language: 'zh',
})

// 标签页配置
const tabs = [
  { key: 'notifications', label: t('settings.notifications.title'), icon: Bell },
  { key: 'privacy', label: t('settings.privacy.title'), icon: View },
  { key: 'messages', label: t('settings.messages.title'), icon: ChatLineSquare },
  { key: 'appearance', label: t('settings.appearance.title'), icon: Brush },
]

/**
 * 返回应用
 */
function goBack() {
  router.push('/app')
}
</script>

<template>
  <div class="settings-layout">
    <!-- 窄边导航栏 -->
    <NavBar>
      <template #quick-bar>
        <QuickBar :items="quickItems" />
      </template>
    </NavBar>

    <!-- 侧边栏 -->
    <aside class="settings-sidebar">
      <div class="sidebar-header">
        <a href="#" @click.prevent="goBack" class="back-link">
          <el-icon><ArrowLeft /></el-icon>
          {{ t('common.back') }}
        </a>
      </div>

      <div class="settings-nav">
        <div
          v-for="tab in tabs"
          :key="tab.key"
          class="settings-nav-item"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          <el-icon :size="18">
            <component :is="tab.icon" />
          </el-icon>
          <span>{{ tab.label }}</span>
        </div>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="settings-main">
      <!-- 通知设置 -->
      <div v-if="activeTab === 'notifications'" class="settings-section">
        <h1 class="settings-title">{{ t('settings.notifications.title') }}</h1>
        <p class="settings-description">{{ t('settings.notifications.description') }}</p>

        <div class="settings-cards">
          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('settings.notifications.pushNotifications') }}</span>
                <el-switch v-model="notificationSettings.enabled" />
              </div>
            </template>
            <div class="setting-options">
              <div class="setting-option">
                <div class="option-info">
                  <span class="option-label">{{ t('profile.preferences.notifications.title') }}</span>
                  <span class="option-desc">{{ t('profile.preferences.notifications.description') }}</span>
                </div>
                <el-switch v-model="notificationSettings.mentions" :disabled="!notificationSettings.enabled" />
              </div>
              <div class="setting-option">
                <div class="option-info">
                  <span class="option-label">{{ t('settings.notifications.replies') }}</span>
                  <span class="option-desc">{{ t('settings.notifications.repliesDesc') }}</span>
                </div>
                <el-switch v-model="notificationSettings.replies" :disabled="!notificationSettings.enabled" />
              </div>
              <div class="setting-option">
                <div class="option-info">
                  <span class="option-label">{{ t('settings.notifications.directMessages') }}</span>
                  <span class="option-desc">{{ t('settings.notifications.directMessagesDesc') }}</span>
                </div>
                <el-switch v-model="notificationSettings.directMessages" :disabled="!notificationSettings.enabled" />
              </div>
              <div class="setting-option">
                <div class="option-info">
                  <span class="option-label">{{ t('settings.notifications.roomInvites') }}</span>
                  <span class="option-desc">{{ t('settings.notifications.roomInvitesDesc') }}</span>
                </div>
                <el-switch v-model="notificationSettings.roomInvites" :disabled="!notificationSettings.enabled" />
              </div>
            </div>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('profile.preferences.soundEffects.title') }}</span>
                <el-switch v-model="notificationSettings.soundEffects" />
              </div>
            </template>
            <p class="card-desc">{{ t('profile.preferences.soundEffects.description') }}</p>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('profile.preferences.messagePreviews.title') }}</span>
                <el-switch v-model="notificationSettings.messagePreview" />
              </div>
            </template>
            <p class="card-desc">{{ t('profile.preferences.messagePreviews.description') }}</p>
          </el-card>
        </div>
      </div>

      <!-- 隐私设置 -->
      <div v-if="activeTab === 'privacy'" class="settings-section">
        <h1 class="settings-title">{{ t('settings.privacy.title') }}</h1>
        <p class="settings-description">{{ t('settings.privacy.description') }}</p>

        <div class="settings-cards">
          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('profile.preferences.onlineStatus.title') }}</span>
                <el-switch v-model="privacySettings.showOnlineStatus" />
              </div>
            </template>
            <p class="card-desc">{{ t('profile.preferences.onlineStatus.description') }}</p>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('settings.privacy.friendRequests') }}</span>
                <el-switch v-model="privacySettings.allowFriendRequests" />
              </div>
            </template>
            <p class="card-desc">{{ t('settings.privacy.friendRequestsDesc') }}</p>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('settings.privacy.roomInvites') }}</span>
                <el-switch v-model="privacySettings.allowRoomInvites" />
              </div>
            </template>
            <p class="card-desc">{{ t('settings.privacy.roomInvitesDesc') }}</p>
          </el-card>
        </div>
      </div>

      <!-- 消息设置 -->
      <div v-if="activeTab === 'messages'" class="settings-section">
        <h1 class="settings-title">{{ t('settings.messages.title') }}</h1>
        <p class="settings-description">{{ t('settings.messages.description') }}</p>

        <div class="settings-cards">
          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('settings.messages.enterToSend') }}</span>
                <el-switch v-model="messageSettings.enterToSend" />
              </div>
            </template>
            <p class="card-desc">{{ t('settings.messages.enterToSendDesc') }}</p>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('settings.messages.showTimestamps') }}</span>
                <el-switch v-model="messageSettings.showTimestamps" />
              </div>
            </template>
            <p class="card-desc">{{ t('settings.messages.showTimestampsDesc') }}</p>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <div class="card-header">
                <span>{{ t('settings.messages.compactMode') }}</span>
                <el-switch v-model="messageSettings.compactMode" />
              </div>
            </template>
            <p class="card-desc">{{ t('settings.messages.compactModeDesc') }}</p>
          </el-card>
        </div>
      </div>

      <!-- 外观设置 -->
      <div v-if="activeTab === 'appearance'" class="settings-section">
        <h1 class="settings-title">{{ t('settings.appearance.title') }}</h1>
        <p class="settings-description">{{ t('settings.appearance.description') }}</p>

        <div class="settings-cards">
          <el-card class="setting-card" shadow="never">
            <template #header>
              <span>{{ t('profile.appearance.theme.title') }}</span>
            </template>
            <div class="theme-options">
              <div
                class="theme-option"
                :class="{ active: themeSettings.theme === 'dark' }"
                @click="themeSettings.theme = 'dark'"
              >
                <div class="theme-preview dark"></div>
                <span>{{ t('profile.appearance.theme.dark') }}</span>
              </div>
              <div
                class="theme-option"
                :class="{ active: themeSettings.theme === 'light' }"
                @click="themeSettings.theme = 'light'"
              >
                <div class="theme-preview light"></div>
                <span>{{ t('profile.appearance.theme.light') }}</span>
              </div>
              <div
                class="theme-option"
                :class="{ active: themeSettings.theme === 'system' }"
                @click="themeSettings.theme = 'system'"
              >
                <div class="theme-preview system"></div>
                <span>{{ t('profile.appearance.theme.system') }}</span>
              </div>
            </div>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <span>{{ t('profile.appearance.messageDensity.title') }}</span>
            </template>
            <el-radio-group v-model="themeSettings.messageDensity" class="density-options">
              <el-radio-button label="comfortable">
                {{ t('profile.appearance.messageDensity.comfortable') }}
              </el-radio-button>
              <el-radio-button label="compact">
                {{ t('profile.appearance.messageDensity.compact') }}
              </el-radio-button>
            </el-radio-group>
          </el-card>

          <el-card class="setting-card" shadow="never">
            <template #header>
              <span>{{ t('profile.appearance.language.title') }}</span>
            </template>
            <el-select v-model="themeSettings.language" class="language-select">
              <el-option label="简体中文" value="zh" />
              <el-option label="English" value="en" />
              <el-option label="日本語" value="ja" />
            </el-select>
          </el-card>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped lang="scss">
.settings-layout {
  display: flex;
  height: 100vh;
  background: var(--wave-bg);
  color: var(--wave-fg);
  overflow: hidden;
}

.settings-sidebar {
  width: 260px;
  min-width: 260px;
  background: var(--wave-sidebar-bg);
  border-right: 1px solid var(--wave-border);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  height: 52px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--wave-border);
}

.back-link {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--wave-muted);
  font-size: 14px;
  text-decoration: none;

  &:hover {
    color: var(--wave-fg);
  }
}

.settings-nav {
  flex: 1;
  padding: 8px;
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--wave-radius);
  font-size: 15px;
  color: var(--wave-muted);
  cursor: pointer;
  transition: all 0.15s;
  margin-bottom: 2px;

  &:hover {
    background: var(--wave-message-hover);
    color: var(--wave-fg);
  }

  &.active {
    background: var(--wave-accent-soft);
    color: var(--wave-fg);
  }
}

.settings-main {
  flex: 1;
  overflow-y: auto;
  padding: 40px 48px;
}

.settings-section {
  max-width: 640px;
}

.settings-title {
  font-family: var(--wave-font-display);
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 8px;
}

.settings-description {
  color: var(--wave-muted);
  font-size: 15px;
  margin: 0 0 32px;
}

.settings-cards {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-card {
  background: var(--wave-surface);
  border: 1px solid var(--wave-border);
  border-radius: var(--wave-radius-lg);

  :deep(.el-card__header) {
    padding: 16px 20px;
    border-bottom: 1px solid var(--wave-border);
  }

  :deep(.el-card__body) {
    padding: 20px;
  }
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 16px;
  font-weight: 600;
}

.card-desc {
  color: var(--wave-muted);
  font-size: 14px;
  margin: 0;
}

.setting-options {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.option-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-label {
  font-size: 14px;
  font-weight: 500;
}

.option-desc {
  font-size: 13px;
  color: var(--wave-muted);
}

.theme-options {
  display: flex;
  gap: 16px;
}

.theme-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-radius: var(--wave-radius);
  border: 2px solid transparent;
  background: var(--wave-bg);
  cursor: pointer;
  transition: all 0.15s;

  &:hover {
    border-color: var(--wave-border);
  }

  &.active {
    border-color: var(--wave-accent);
  }
}

.theme-preview {
  width: 80px;
  height: 60px;
  border-radius: var(--wave-radius);
  border: 1px solid var(--wave-border);

  &.dark {
    background: #0b0b14;
  }

  &.light {
    background: #f5f5f7;
  }

  &.system {
    background: linear-gradient(135deg, #0b0b14 50%, #f5f5f7 50%);
  }
}

.density-options {
  width: 100%;
}

.language-select {
  width: 100%;
}
</style>
