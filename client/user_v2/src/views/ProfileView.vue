<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { useRoomStore } from '@/stores/room'
import { useSettingsStore } from '@/stores/settings'
import { uploadApi } from '@/api/upload'
import { userApi } from '@/api/user'
import type { LocaleSettings } from '@/types/settings'
import {
  Clock,
  User,
  Message,
  Share,
  Star,
  Delete,
  SwitchButton,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t, locale } = useI18n()
const authStore = useAuthStore()
const roomStore = useRoomStore()
const settingsStore = useSettingsStore()

// 加载状态
const loading = ref(false)

// 当前用户
const currentUser = computed(() => authStore.user)

// 头像上传
const uploadingAvatar = ref(false)
const avatarInputRef = ref<HTMLInputElement | null>(null)

function triggerAvatarUpload() {
  avatarInputRef.value?.click()
}

async function handleAvatarSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  input.value = ''

  uploadingAvatar.value = true
  try {
    const res = await uploadApi.uploadAvatar(file)
    if (res.success && res.data) {
      await authStore.fetchUser()
      ElMessage.success('Avatar updated')
    } else {
      ElMessage.error(res.message || t('common.error'))
    }
  } catch {
    ElMessage.error(t('common.error'))
  } finally {
    uploadingAvatar.value = false
  }
}

// 用户统计
const userStats = computed(() => ({
  joinedRooms: roomStore.rooms.length,
  joinedDate: currentUser.value?.created_at
    ? new Date(currentUser.value.created_at).toLocaleDateString(locale.value === 'zh' ? 'zh-CN' : 'en-US', { year: 'numeric', month: 'long' })
    : '-',
}))

// ========== 通知设置 ==========
const notificationSettings = computed(() => settingsStore.notificationSettings)

async function updateNotification(key: keyof typeof notificationSettings.value, value: boolean) {
  // 乐观更新：只传递修改的字段，store 会立即更新本地状态并同步到服务端
  const result = await settingsStore.updateNotificationSettings({ [key]: value } as Partial<typeof notificationSettings.value>)
  if (!result.success) {
    console.error('[ProfileView] Update notification failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// ========== 隐私设置 ==========
const privacySettings = computed(() => settingsStore.privacySettings)

async function updatePrivacy(key: keyof typeof privacySettings.value, value: string | boolean) {
  // 乐观更新：只传递修改的字段
  const result = await settingsStore.updatePrivacySettings({ [key]: value } as Partial<typeof privacySettings.value>)
  if (!result.success) {
    console.error('[ProfileView] Update privacy failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// ========== 消息设置 ==========
const messageSettings = computed(() => settingsStore.messageSettings)

async function updateMessage(key: keyof typeof messageSettings.value, value: boolean) {
  // 乐观更新：只传递修改的字段
  const result = await settingsStore.updateMessageSettings({ [key]: value } as Partial<typeof messageSettings.value>)
  if (!result.success) {
    console.error('[ProfileView] Update message failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// ========== 安全设置 ==========
const securitySettings = computed(() => settingsStore.securitySettings)

async function updateSecurity(key: keyof typeof securitySettings.value, value: boolean) {
  // 乐观更新：只传递修改的字段
  const result = await settingsStore.updateSecuritySettings({ [key]: value } as Partial<typeof securitySettings.value>)
  if (!result.success) {
    console.error('[ProfileView] Update security failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// ========== 语言与地区设置 ==========
async function updateLocale(key: keyof LocaleSettings, value: string) {
  // 如果修改了语言，立即更新 i18n locale（乐观更新）
  if (key === 'language') {
    const newI18nLocale = value === 'zh-CN' ? 'zh' : value === 'ja-JP' ? 'ja' : 'en'
    locale.value = newI18nLocale
    localStorage.setItem('locale', newI18nLocale)
  }

  // 乐观更新：只传递修改的字段
  const result = await settingsStore.updateLocaleSettings({ [key]: value } as Partial<LocaleSettings>)
  if (!result.success) {
    console.error('[ProfileView] Update locale failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// ========== 无障碍设置 ==========
const accessibilitySettings = computed(() => settingsStore.accessibilitySettings)

async function updateAccessibility(key: keyof typeof accessibilitySettings.value, value: string | boolean) {
  // 乐观更新：只传递修改的字段
  const result = await settingsStore.updateAccessibilitySettings({ [key]: value } as Partial<typeof accessibilitySettings.value>)
  if (!result.success) {
    console.error('[ProfileView] Update accessibility failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// ========== 媒体设置 ==========
const mediaSettings = computed(() => settingsStore.mediaSettings)

async function updateMedia(key: keyof typeof mediaSettings.value, value: boolean | string) {
  // 乐观更新：只传递修改的字段
  const result = await settingsStore.updateMediaSettings({ [key]: value } as Partial<typeof mediaSettings.value>)
  if (!result.success) {
    console.error('[ProfileView] Update media failed:', result)
    const errorMsg = typeof result.error === 'string' ? result.error.trim() : '保存失败，请重试'
    ElMessage.error({ message: errorMsg, duration: 3000 })
  }
}

// 初始化
onMounted(async () => {
  loading.value = true
  await authStore.fetchUser()
  await roomStore.fetchMyRooms()
  await settingsStore.loadSettings()

  loading.value = false
})

/**
 * 获取用户首字母
 */
function getInitials(name: string): string {
  return name ? name.charAt(0).toUpperCase() : '?'
}

/**
 * 发送消息
 */
function handleMessage() {
  router.push('/app')
}

/**
 * 分享个人资料
 */
function handleShare() {
  if (currentUser.value?.username) {
    navigator.clipboard.writeText(`${window.location.origin}/profile/${currentUser.value.username}`)
    ElMessage.success(t('profile.shareCopied'))
  }
}

/**
 * 退出登录
 */
async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}

/**
 * 删除账户
 */
function deleteAccount() {
  ElMessageBox.confirm(
    t('profile.dangerZone.confirmDelete'),
    t('profile.dangerZone.deleteAccount'),
    {
      confirmButtonText: t('common.delete'),
      cancelButtonText: t('common.cancel'),
      type: 'warning',
    }
  ).then(async () => {
    try {
      await userApi.deleteAccount()
      await authStore.logout()
      ElMessage.success(t('profile.dangerZone.accountDeleted'))
      router.push('/')
    } catch {
      ElMessage.error(t('common.error'))
    }
  }).catch(() => {})
}

// 选项数据
const languageOptions = [
  { label: 'English', value: 'en-US' },
  { label: '简体中文', value: 'zh-CN' },
  { label: '日本語', value: 'ja-JP' },
]

const timezoneOptions = [
  { label: 'Asia/Shanghai (UTC+8)', value: 'Asia/Shanghai' },
  { label: 'Asia/Tokyo (UTC+9)', value: 'Asia/Tokyo' },
  { label: 'America/New_York (UTC-5)', value: 'America/New_York' },
  { label: 'Europe/London (UTC+0)', value: 'Europe/London' },
]

const timeFormatOptions = [
  { label: '24小时制', value: '24h' },
  { label: '12小时制', value: '12h' },
]

const dateFormatOptions = [
  { label: 'YYYY-MM-DD', value: 'YYYY-MM-DD' },
  { label: 'DD/MM/YYYY', value: 'DD/MM/YYYY' },
  { label: 'MM/DD/YYYY', value: 'MM/DD/YYYY' },
]

const visibilityOptions = [
  { label: '所有人', value: 'everyone' },
  { label: '仅好友', value: 'friends' },
  { label: '不可见', value: 'none' },
]

const fontSizeOptions = [
  { label: '小', value: 'small' },
  { label: '中', value: 'medium' },
  { label: '大', value: 'large' },
]

const imageQualityOptions = [
  { label: '原图', value: 'original' },
  { label: '高', value: 'high' },
  { label: '中', value: 'medium' },
  { label: '低', value: 'low' },
]
</script>

<template>
  <div class="profile-layout">
    <!-- 主内容区 -->
    <main class="profile-main">
      <!-- 加载状态 -->
      <div v-if="loading" class="container loading">
        <el-skeleton :rows="10" animated />
      </div>

      <div v-else class="container">
        <!-- 个人资料头部 -->
        <div class="profile-header">
          <div
            class="profile-avatar"
            :class="{ 'profile-avatar--uploading': uploadingAvatar }"
            @click="triggerAvatarUpload"
          >
            <template v-if="currentUser?.avatar_url">
              <img :src="currentUser.avatar_url" :alt="currentUser.username" class="profile-avatar__img" />
            </template>
            <template v-else>
              <span>{{ getInitials(currentUser?.username || '') }}</span>
            </template>
            <div class="profile-avatar__overlay">
              <span v-if="uploadingAvatar">…</span>
              <span v-else>Change</span>
            </div>
            <span class="status-big"></span>
            <input
              ref="avatarInputRef"
              type="file"
              accept="image/*"
              style="display:none"
              @change="handleAvatarSelected"
            />
          </div>
          <div class="profile-info">
            <h1>{{ currentUser?.username || 'User' }}</h1>
            <p class="handle">@{{ currentUser?.username }} · {{ currentUser?.email }}</p>
            <div class="profile-meta">
              <span>
                <el-icon><Clock /></el-icon>
                {{ t('profile.joined') }} {{ userStats.joinedDate }}
              </span>
              <span>
                <el-icon><User /></el-icon>
                {{ userStats.joinedRooms }} {{ t('profile.mutualServers') }}
              </span>
              <span v-if="currentUser?.role === 'admin' || currentUser?.role === 'super_admin'">
                <el-icon><Star /></el-icon>
                {{ t('profile.admin') }}
              </span>
            </div>
            <div class="profile-actions">
              <el-button type="primary" @click="handleMessage">
                <el-icon><Message /></el-icon>
                {{ t('profile.message') }}
              </el-button>
              <el-button @click="handleShare">
                <el-icon><Share /></el-icon>
                {{ t('profile.shareProfile') }}
              </el-button>
            </div>
          </div>
        </div>

        <!-- 基本信息 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.basicInfo.title') }}</div>
          <div class="info-item">
            <span class="info-label">{{ t('profile.settings.basicInfo.username') }}</span>
            <span class="info-value">{{ currentUser?.username }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ t('profile.settings.basicInfo.email') }}</span>
            <span class="info-value">{{ currentUser?.email }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ t('profile.settings.basicInfo.role') }}</span>
            <span class="info-value">{{ currentUser?.role || 'user' }}</span>
          </div>
        </div>

        <!-- 通知设置 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.notifications.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.enable') }}</h3>
              <p>{{ t('profile.settings.notifications.enableDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableNotification }"
              @click="updateNotification('enableNotification', !notificationSettings.enableNotification)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.directMessage') }}</h3>
              <p>{{ t('profile.settings.notifications.directMessageDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableDirectMessage }"
              :disabled="!notificationSettings.enableNotification"
              @click="updateNotification('enableDirectMessage', !notificationSettings.enableDirectMessage)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.mention') }}</h3>
              <p>{{ t('profile.settings.notifications.mentionDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableMention }"
              :disabled="!notificationSettings.enableNotification"
              @click="updateNotification('enableMention', !notificationSettings.enableMention)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.roomInvitation') }}</h3>
              <p>{{ t('profile.settings.notifications.roomInvitationDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableRoomInvitation }"
              :disabled="!notificationSettings.enableNotification"
              @click="updateNotification('enableRoomInvitation', !notificationSettings.enableRoomInvitation)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.system') }}</h3>
              <p>{{ t('profile.settings.notifications.systemDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableSystemNotification }"
              :disabled="!notificationSettings.enableNotification"
              @click="updateNotification('enableSystemNotification', !notificationSettings.enableSystemNotification)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.sound') }}</h3>
              <p>{{ t('profile.settings.notifications.soundDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableSound }"
              @click="updateNotification('enableSound', !notificationSettings.enableSound)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.notifications.desktop') }}</h3>
              <p>{{ t('profile.settings.notifications.desktopDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: notificationSettings.enableDesktopNotification }"
              @click="updateNotification('enableDesktopNotification', !notificationSettings.enableDesktopNotification)"
            ></button>
          </div>
        </div>

        <!-- 隐私设置 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.privacy.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.privacy.onlineStatus') }}</h3>
              <p>{{ t('profile.settings.privacy.onlineStatusDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="privacySettings.onlineStatusVisibility"
              @change="updatePrivacy('onlineStatusVisibility', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in visibilityOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.privacy.profile') }}</h3>
              <p>{{ t('profile.settings.privacy.profileDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="privacySettings.profileVisibility"
              @change="updatePrivacy('profileVisibility', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in visibilityOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.privacy.strangerMessage') }}</h3>
              <p>{{ t('profile.settings.privacy.strangerMessageDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: privacySettings.allowStrangerMessage }"
              @click="updatePrivacy('allowStrangerMessage', !privacySettings.allowStrangerMessage)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.privacy.roomInvite') }}</h3>
              <p>{{ t('profile.settings.privacy.roomInviteDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: privacySettings.allowRoomInvitation }"
              @click="updatePrivacy('allowRoomInvitation', !privacySettings.allowRoomInvitation)"
            ></button>
          </div>
        </div>

        <!-- 消息设置 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.message.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.message.preview') }}</h3>
              <p>{{ t('profile.settings.message.previewDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: messageSettings.showMessagePreview }"
              @click="updateMessage('showMessagePreview', !messageSettings.showMessagePreview)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.message.readReceipt') }}</h3>
              <p>{{ t('profile.settings.message.readReceiptDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: messageSettings.enableReadReceipt }"
              @click="updateMessage('enableReadReceipt', !messageSettings.enableReadReceipt)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.message.typing') }}</h3>
              <p>{{ t('profile.settings.message.typingDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: messageSettings.showTypingStatus }"
              @click="updateMessage('showTypingStatus', !messageSettings.showTypingStatus)"
            ></button>
          </div>
        </div>

        <!-- 语言与地区 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.locale.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.locale.language') }}</h3>
              <p>{{ t('profile.settings.locale.languageDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="settingsStore.localeSettings.language"
              @change="updateLocale('language', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in languageOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.locale.timezone') }}</h3>
              <p>{{ t('profile.settings.locale.timezoneDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="settingsStore.localeSettings.timezone"
              @change="updateLocale('timezone', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in timezoneOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.locale.timeFormat') }}</h3>
              <p>{{ t('profile.settings.locale.timeFormatDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="settingsStore.localeSettings.timeFormat"
              @change="updateLocale('timeFormat', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in timeFormatOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.locale.dateFormat') }}</h3>
              <p>{{ t('profile.settings.locale.dateFormatDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="settingsStore.localeSettings.dateFormat"
              @change="updateLocale('dateFormat', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in dateFormatOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
        </div>

        <!-- 无障碍设置 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.accessibility.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.accessibility.fontSize') }}</h3>
              <p>{{ t('profile.settings.accessibility.fontSizeDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="accessibilitySettings.fontSize"
              @change="updateAccessibility('fontSize', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in fontSizeOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.accessibility.highContrast') }}</h3>
              <p>{{ t('profile.settings.accessibility.highContrastDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: accessibilitySettings.highContrast }"
              @click="updateAccessibility('highContrast', !accessibilitySettings.highContrast)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.accessibility.reduceMotion') }}</h3>
              <p>{{ t('profile.settings.accessibility.reduceMotionDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: accessibilitySettings.reduceMotion }"
              @click="updateAccessibility('reduceMotion', !accessibilitySettings.reduceMotion)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.accessibility.compactMode') }}</h3>
              <p>{{ t('profile.settings.accessibility.compactModeDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: accessibilitySettings.compactMode }"
              @click="updateAccessibility('compactMode', !accessibilitySettings.compactMode)"
            ></button>
          </div>
        </div>

        <!-- 媒体设置 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.media.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.media.autoDownload') }}</h3>
              <p>{{ t('profile.settings.media.autoDownloadDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: mediaSettings.autoDownloadMedia }"
              @click="updateMedia('autoDownloadMedia', !mediaSettings.autoDownloadMedia)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.media.saveGallery') }}</h3>
              <p>{{ t('profile.settings.media.saveGalleryDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: mediaSettings.saveMediaGallery }"
              @click="updateMedia('saveMediaGallery', !mediaSettings.saveMediaGallery)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.media.imageQuality') }}</h3>
              <p>{{ t('profile.settings.media.imageQualityDesc') }}</p>
            </div>
            <select
              class="select-input"
              :value="mediaSettings.imageQuality"
              @change="updateMedia('imageQuality', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in imageQualityOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.media.autoPlayVideo') }}</h3>
              <p>{{ t('profile.settings.media.autoPlayVideoDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: mediaSettings.autoPlayVideo }"
              @click="updateMedia('autoPlayVideo', !mediaSettings.autoPlayVideo)"
            ></button>
          </div>
        </div>

        <!-- 安全设置 -->
        <div class="section">
          <div class="section-title">{{ t('profile.settings.security.title') }}</div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.security.abnormalLogin') }}</h3>
              <p>{{ t('profile.settings.security.abnormalLoginDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: securitySettings.enableAbnormalLoginAlert }"
              @click="updateSecurity('enableAbnormalLoginAlert', !securitySettings.enableAbnormalLoginAlert)"
            ></button>
          </div>
          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.settings.security.singleDevice') }}</h3>
              <p>{{ t('profile.settings.security.singleDeviceDesc') }}</p>
            </div>
            <button
              class="toggle"
              :class="{ on: securitySettings.enableSingleDeviceLogin }"
              @click="updateSecurity('enableSingleDeviceLogin', !securitySettings.enableSingleDeviceLogin)"
            ></button>
          </div>
        </div>

        <!-- 危险区域 -->
        <div class="danger-zone">
          <h3>{{ t('profile.settings.dangerZone.title') }}</h3>
          <p>{{ t('profile.settings.dangerZone.description') }}</p>
          <div class="danger-actions">
            <el-button type="danger" plain @click="handleLogout">
              <el-icon><SwitchButton /></el-icon>
              {{ t('profile.settings.dangerZone.logout') }}
            </el-button>
            <el-button type="danger" plain @click="deleteAccount">
              <el-icon><Delete /></el-icon>
              {{ t('profile.settings.dangerZone.deleteAccount') }}
            </el-button>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped lang="scss">
.profile-layout {
  display: flex;
  height: 100vh;
  background: var(--bg);
  color: var(--fg);
  overflow: hidden;
}

.profile-main {
  flex: 1;
  overflow-y: auto;
}

.container {
  max-width: 720px;
  margin-inline: auto;
  padding-inline: 24px;

  &.loading {
    padding-top: 40px;
  }
}

// 个人资料头部
.profile-header {
  padding: 40px 0 32px;
  display: flex;
  gap: 28px;
  align-items: center;
  flex-wrap: wrap;
}

.profile-avatar {
  width: 96px;
  height: 96px;
  min-width: 96px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
  display: grid;
  place-items: center;
  font-size: 36px;
  font-weight: 700;
  color: #fff;
  position: relative;
  cursor: pointer;
  overflow: hidden;

  &--uploading {
    opacity: 0.7;
    pointer-events: none;
  }

  &__img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 50%;
  }

  &__overlay {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
    color: #fff;
    font-size: 13px;
    font-weight: 600;
    opacity: 0;
    transition: opacity 0.2s;
    border-radius: 50%;
  }

  &:hover &__overlay {
    opacity: 1;
  }
}

.status-big {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--accent-green);
  border: 3px solid var(--bg);
}

.profile-info {
  flex: 1;

  h1 {
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 600;
    margin-bottom: 4px;
  }
}

.handle {
  font-size: 15px;
  color: var(--muted);
  margin-bottom: 8px;
}

.profile-meta {
  display: flex;
  gap: 24px;
  margin-top: 12px;
  flex-wrap: wrap;

  span {
    font-size: 13px;
    color: var(--muted);
    display: flex;
    align-items: center;
    gap: 6px;

    .el-icon {
      font-size: 14px;
    }
  }
}

.profile-actions {
  display: flex;
  gap: 10px;
  margin-top: 16px;
  flex-wrap: wrap;
}

// 分区
.section {
  border-top: 1px solid var(--border);
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  padding: 24px 0 16px;
}

.pref-group {
  padding: 20px 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;

  & + .pref-group {
    border-top: 1px solid var(--border);
  }
}

.pref-label {
  h3 {
    font-size: 15px;
    font-weight: 500;
  }

  p {
    font-size: 13px;
    color: var(--muted);
    margin-top: 2px;
  }
}

// 自定义开关样式
.toggle {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  background: var(--border);
  border: none;
  cursor: pointer;
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;

  &::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s;
  }

  &.on {
    background: var(--accent);

    &::after {
      transform: translateX(20px);
    }
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

// 自定义选择框样式
.select-input {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  color: var(--fg);
  font: inherit;
  font-size: 14px;
  min-width: 160px;
  cursor: pointer;

  &:focus {
    outline: none;
    border-color: var(--accent);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  option {
    background: var(--surface);
    color: var(--fg);
  }
}

// 信息项
.info-item {
  display: flex;
  justify-content: space-between;
  padding: 16px 0;
  border-bottom: 1px solid var(--border);

  &:last-child {
    border-bottom: none;
  }
}

.info-label {
  font-size: 15px;
  font-weight: 500;
}

.info-value {
  font-size: 14px;
  color: var(--muted);
}

// 危险区域
.danger-zone {
  padding: 20px;
  border: 1px solid color-mix(in oklch, var(--accent-orange) 40%, transparent);
  border-radius: var(--radius-lg);
  margin: 24px 0 48px;

  h3 {
    font-size: 15px;
    font-weight: 600;
    color: var(--accent-orange);
    margin-bottom: 4px;
  }

  p {
    font-size: 13px;
    color: var(--muted);
    margin-bottom: 16px;
  }
}

.danger-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

// 响应式
@media (max-width: 640px) {
  .profile-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .profile-meta {
    gap: 12px;
  }

  .pref-group {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .select-input {
    width: 100%;
  }
}
</style>
