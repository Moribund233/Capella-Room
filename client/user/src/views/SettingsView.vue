<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Component } from 'vue'
import { storeToRefs } from 'pinia'
import { useMessage } from 'naive-ui'
import { useSettingsStore } from '@/stores/settings'
import { useThemeStore } from '@/stores/theme'
import { usePersonalizationStore } from '@/stores/personalization'
import { useGlobalModal } from '@/composables/useGlobalModal'
import PersonalizationModal from '@/components/quick/PersonalizationModal.vue'
import { useResponsive } from '@/composables/useResponsive'
import PageTransition from '@/components/ui/PageTransition.vue'
import type { ThemeType } from '@/stores/theme'

// 设置卡片组件
import NotificationSettingsCard from '@/components/settings/NotificationSettingsCard.vue'
import SecuritySettingsCard from '@/components/settings/SecuritySettingsCard.vue'
import ThemeSettingsCard from '@/components/settings/ThemeSettingsCard.vue'
import PrivacySettingsCard from '@/components/settings/PrivacySettingsCard.vue'
import MessageSettingsCard from '@/components/settings/MessageSettingsCard.vue'
import LocaleSettingsCard from '@/components/settings/LocaleSettingsCard.vue'
import AccessibilitySettingsCard from '@/components/settings/AccessibilitySettingsCard.vue'
import MediaSettingsCard from '@/components/settings/MediaSettingsCard.vue'

import {
  Bell,
  Shield,
  Palette,
  Eye,
  MessageSquare,
  Globe,
  Type,
  Image,
} from 'lucide-vue-next'

const message = useMessage()
const settingsStore = useSettingsStore()
const themeStore = useThemeStore()
const personalizationStore = usePersonalizationStore()
const globalModal = useGlobalModal()
useResponsive()

const {
  notificationSettings,
  privacySettings,
  messageSettings,
  securitySettings,
  localeSettings,
  accessibilitySettings,
  mediaSettings,
  loginDevices,
  securityLoading,
} = storeToRefs(settingsStore)

const activeTab = ref('notifications')
const savingTab = ref<string | null>(null)

/**
 * Tab 配置
 */
interface TabConfig {
  key: string
  label: string
  icon: Component
}

const tabs: TabConfig[] = [
  { key: 'notifications', label: '通知', icon: Bell },
  { key: 'security', label: '账号安全', icon: Shield },
  { key: 'theme', label: '主题', icon: Palette },
  { key: 'privacy', label: '隐私', icon: Eye },
  { key: 'messages', label: '消息', icon: MessageSquare },
  { key: 'locale', label: '语言', icon: Globe },
  { key: 'accessibility', label: '无障碍', icon: Type },
  { key: 'media', label: '媒体', icon: Image },
]

/**
 * 保存通知设置
 */
async function saveNotificationSettings(settings: typeof notificationSettings.value) {
  savingTab.value = 'notifications'
  const success = await settingsStore.updateNotificationSettings(settings)
  if (success) {
    message.success('通知设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 保存隐私设置
 */
async function savePrivacySettings(settings: typeof privacySettings.value) {
  savingTab.value = 'privacy'
  const success = await settingsStore.updatePrivacySettings(settings)
  if (success) {
    message.success('隐私设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 保存消息设置
 */
async function saveMessageSettings(settings: typeof messageSettings.value) {
  savingTab.value = 'messages'
  const success = await settingsStore.updateMessageSettings(settings)
  if (success) {
    message.success('消息设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 保存安全设置
 */
async function saveSecuritySettings(settings: typeof securitySettings.value) {
  savingTab.value = 'security'
  const success = await settingsStore.updateSecuritySettings(settings)
  if (success) {
    message.success('安全设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 保存语言设置
 */
async function saveLocaleSettings(settings: typeof localeSettings.value) {
  savingTab.value = 'locale'
  const success = await settingsStore.updateLocaleSettings(settings)
  if (success) {
    message.success('语言设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 保存无障碍设置
 */
async function saveAccessibilitySettings(settings: typeof accessibilitySettings.value) {
  savingTab.value = 'accessibility'
  const success = await settingsStore.updateAccessibilitySettings(settings)
  if (success) {
    message.success('无障碍设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 保存媒体设置
 */
async function saveMediaSettings(settings: typeof mediaSettings.value) {
  savingTab.value = 'media'
  const success = await settingsStore.updateMediaSettings(settings)
  if (success) {
    message.success('媒体设置已保存')
  } else {
    message.error('保存失败，请重试')
  }
  savingTab.value = null
}

/**
 * 设置主题
 */
function setTheme(theme: ThemeType) {
  themeStore.setTheme(theme)
  personalizationStore.updateConfig({ theme })
}

/**
 * 打开个性化设置弹窗
 */
function openPersonalizationModal() {
  globalModal.open({
    title: '个性化设置',
    component: PersonalizationModal,
    componentProps: {
      modelValue: personalizationStore.config,
      'onUpdate:modelValue': (value: typeof personalizationStore.config) => {
        if (value.theme !== themeStore.themeSetting) {
          themeStore.setTheme(value.theme)
        }
        personalizationStore.updateConfig(value)
      },
      onConfirm: () => {
        globalModal.close()
      },
      onReset: () => {
        themeStore.setTheme('system')
        personalizationStore.resetToDefault()
      },
    },
    preset: 'card',
    maskClosable: true,
    closable: true,
  })
}

/**
 * 登出设备
 */
async function logoutDevice(deviceId: string) {
  const success = await settingsStore.logoutDevice(deviceId)
  if (success) {
    message.success('设备已登出')
  } else {
    message.error('操作失败，请重试')
  }
}

/**
 * 禁用设备
 */
async function blockDevice(deviceId: string) {
  const success = await settingsStore.blockDevice(deviceId)
  if (success) {
    message.success('设备已禁用')
  } else {
    message.error('操作失败，请重试')
  }
}

/**
 * 启用设备
 */
async function unblockDevice(deviceId: string) {
  const success = await settingsStore.unblockDevice(deviceId)
  if (success) {
    message.success('设备已启用')
  } else {
    message.error('操作失败，请重试')
  }
}

/**
 * 清除缓存
 */
function clearCache() {
  message.success('缓存已清除')
}

onMounted(async () => {
  await settingsStore.ensureLoaded()
})
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <h1 class="page-title">设置</h1>
    </div>

    <div class="settings-container">
      <!-- 左侧 Tab 导航 -->
      <div class="settings-sidebar">
        <div
          v-for="tab in tabs"
          :key="tab.key"
          class="settings-tab-item"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          <component :is="tab.icon" :size="18" class="tab-icon" />
          <span class="tab-label">{{ tab.label }}</span>
        </div>
      </div>

      <!-- 右侧内容区域 -->
      <div class="settings-content">
        <PageTransition name="tab-slide" mode="out-in" :duration="250">
          <!-- 通知设置 -->
          <div v-if="activeTab === 'notifications'" key="notifications" class="settings-panel">
            <NotificationSettingsCard
              v-model="notificationSettings"
              :saving="savingTab === 'notifications'"
              @save="saveNotificationSettings"
            />
          </div>

          <!-- 账号安全 -->
          <div v-else-if="activeTab === 'security'" key="security" class="settings-panel">
            <SecuritySettingsCard
              v-model="securitySettings"
              :devices="loginDevices"
              :loading="securityLoading"
              :device-loading="securityLoading"
              @save="saveSecuritySettings"
              @logout-device="logoutDevice"
              @block-device="blockDevice"
              @unblock-device="unblockDevice"
            />
          </div>

          <!-- 主题设置 -->
          <div v-else-if="activeTab === 'theme'" key="theme" class="settings-panel">
            <ThemeSettingsCard
              v-model="themeStore.themeSetting"
              @update:model-value="setTheme"
              @open-personalization="openPersonalizationModal"
            />
          </div>

          <!-- 隐私设置 -->
          <div v-else-if="activeTab === 'privacy'" key="privacy" class="settings-panel">
            <PrivacySettingsCard
              v-model="privacySettings"
              :saving="savingTab === 'privacy'"
              @save="savePrivacySettings"
            />
          </div>

          <!-- 消息设置 -->
          <div v-else-if="activeTab === 'messages'" key="messages" class="settings-panel">
            <MessageSettingsCard
              v-model="messageSettings"
              :saving="savingTab === 'messages'"
              @save="saveMessageSettings"
            />
          </div>

          <!-- 语言与地区 -->
          <div v-else-if="activeTab === 'locale'" key="locale" class="settings-panel">
            <LocaleSettingsCard
              v-model="localeSettings"
              :saving="savingTab === 'locale'"
              @save="saveLocaleSettings"
            />
          </div>

          <!-- 无障碍设置 -->
          <div v-else-if="activeTab === 'accessibility'" key="accessibility" class="settings-panel">
            <AccessibilitySettingsCard
              v-model="accessibilitySettings"
              :saving="savingTab === 'accessibility'"
              @save="saveAccessibilitySettings"
            />
          </div>

          <!-- 媒体与存储 -->
          <div v-else-if="activeTab === 'media'" key="media" class="settings-panel">
            <MediaSettingsCard
              v-model="mediaSettings"
              :saving="savingTab === 'media'"
              @save="saveMediaSettings"
              @clear-cache="clearCache"
            />
          </div>
        </PageTransition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  margin-bottom: 24px;
  flex-shrink: 0;
}

.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.settings-container {
  display: flex;
  flex: 1;
  overflow: hidden;
  background: var(--color-white);
  border-radius: 12px;
  border: 1px solid var(--color-border);
}

/* 左侧侧边栏 */
.settings-sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-background-light);
  border-right: 1px solid var(--color-border);
  padding: 16px 12px;
  overflow-y: auto;
}

.settings-tab-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--color-text-secondary);
  margin-bottom: 4px;
  position: relative;
  overflow: hidden;
}

.settings-tab-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%) scaleY(0);
  width: 3px;
  height: 20px;
  background: var(--color-primary);
  border-radius: 0 2px 2px 0;
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-tab-item:hover {
  background: var(--color-primary-soft);
  color: var(--color-primary);
  transform: translateX(2px);
}

.settings-tab-item.active {
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-weight: 500;
  transform: translateX(4px);
}

.settings-tab-item.active::before {
  transform: translateY(-50%) scaleY(1);
}

.settings-tab-item .tab-icon {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-tab-item:hover .tab-icon,
.settings-tab-item.active .tab-icon {
  transform: scale(1.1);
}

.tab-icon {
  flex-shrink: 0;
}

.tab-label {
  font-size: 14px;
}

/* 右侧内容区域 */
.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background: var(--color-white);
}

.settings-panel {
  height: 100%;
}

/* 自定义滚动条 */
.settings-content::-webkit-scrollbar,
.settings-sidebar::-webkit-scrollbar {
  width: 6px;
}

.settings-content::-webkit-scrollbar-track,
.settings-sidebar::-webkit-scrollbar-track {
  background: transparent;
}

.settings-content::-webkit-scrollbar-thumb,
.settings-sidebar::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.settings-content::-webkit-scrollbar-thumb:hover,
.settings-sidebar::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}

/* 移动端适配 */
@media (max-width: 768px) {
  .settings-view {
    padding: 16px;
  }

  .settings-header {
    margin-bottom: 16px;
  }

  .page-title {
    font-size: 20px;
  }

  .settings-container {
    flex-direction: column;
  }

  .settings-sidebar {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 8px;
    display: flex;
    gap: 8px;
    overflow-x: auto;
    overflow-y: hidden;
  }

  .settings-tab-item {
    flex-direction: column;
    gap: 4px;
    padding: 8px 12px;
    margin-bottom: 0;
    min-width: 64px;
    text-align: center;
  }

  .tab-label {
    font-size: 12px;
  }

  .settings-content {
    padding: 16px;
  }
}
</style>
