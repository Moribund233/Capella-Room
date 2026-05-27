<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { useRoomStore } from '@/stores/room'
import { useSettingsStore } from '@/stores/settings'
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
const saving = ref(false)

// 当前用户
const currentUser = computed(() => authStore.user)

// 用户统计
const userStats = computed(() => ({
  joinedRooms: roomStore.rooms.length,
  joinedDate: currentUser.value?.created_at
    ? new Date(currentUser.value.created_at).toLocaleDateString(locale.value === 'zh' ? 'zh-CN' : 'en-US', { year: 'numeric', month: 'long' })
    : '-',
}))

// 设置项 - 使用 settings store
const preferences = computed({
  get: () => ({
    notifications: settingsStore.notificationSettings.enableNotification,
    messagePreview: settingsStore.notificationSettings.enableDesktopNotification,
    soundEffects: settingsStore.notificationSettings.enableSound,
    showOnlineStatus: settingsStore.privacySettings.onlineStatusVisibility !== 'none',
  }),
  set: async (val) => {
    saving.value = true
    await settingsStore.updateSettings({
      notifications: {
        ...settingsStore.notificationSettings,
        enableNotification: val.notifications,
        enableDesktopNotification: val.messagePreview,
        enableSound: val.soundEffects,
      },
      privacy: {
        ...settingsStore.privacySettings,
        onlineStatusVisibility: val.showOnlineStatus ? 'everyone' : 'none',
      },
    })
    saving.value = false
  },
})

const appearance = computed({
  get: () => ({
    theme: settingsStore.localeSettings.language === 'zh-CN' ? 'Dark' : 'Dark',
    messageDensity: settingsStore.accessibilitySettings.compactMode ? 'Compact' : 'Comfortable',
    language: settingsStore.localeSettings.language === 'zh-CN' ? 'Chinese' : 'English',
  }),
  set: async (val) => {
    saving.value = true
    const newLocale = val.language === 'Chinese' ? 'zh-CN' : val.language === 'Japanese' ? 'ja-JP' : 'en-US'
    locale.value = newLocale === 'zh-CN' ? 'zh' : newLocale === 'ja-JP' ? 'ja' : 'en'
    
    await settingsStore.updateSettings({
      locale: {
        ...settingsStore.localeSettings,
        language: newLocale,
      },
      accessibility: {
        ...settingsStore.accessibilitySettings,
        compactMode: val.messageDensity === 'Compact',
      },
    })
    saving.value = false
  },
})

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
      await authStore.logout()
      ElMessage.success(t('common.success'))
      router.push('/')
    } catch {
      ElMessage.error(t('common.error'))
    }
  }).catch(() => {})
}

// 语言选项
const languageOptions = [
  { label: 'English', value: 'English' },
  { label: '简体中文', value: 'Chinese' },
  { label: '日本語', value: 'Japanese' },
]

// 主题选项
const themeOptions = [
  { label: t('profile.appearance.theme.dark'), value: 'Dark' },
  { label: t('profile.appearance.theme.light'), value: 'Light' },
  { label: t('profile.appearance.theme.system'), value: 'System' },
]

// 密度选项
const densityOptions = [
  { label: t('profile.appearance.density.comfortable'), value: 'Comfortable' },
  { label: t('profile.appearance.density.compact'), value: 'Compact' },
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
          <div class="profile-avatar">
            <span>{{ getInitials(currentUser?.username || '') }}</span>
            <span class="status-big"></span>
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

        <!-- Preferences -->
        <div class="section">
          <div class="section-title">{{ t('profile.preferences.title') }}</div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.preferences.notifications.title') }}</h3>
              <p>{{ t('profile.preferences.notifications.description') }}</p>
            </div>
            <el-switch v-model="preferences.notifications" :loading="saving" />
          </div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.preferences.messagePreviews.title') }}</h3>
              <p>{{ t('profile.preferences.messagePreviews.description') }}</p>
            </div>
            <el-switch v-model="preferences.messagePreview" :loading="saving" />
          </div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.preferences.soundEffects.title') }}</h3>
              <p>{{ t('profile.preferences.soundEffects.description') }}</p>
            </div>
            <el-switch v-model="preferences.soundEffects" :loading="saving" />
          </div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.preferences.onlineStatus.title') }}</h3>
              <p>{{ t('profile.preferences.onlineStatus.description') }}</p>
            </div>
            <el-switch v-model="preferences.showOnlineStatus" :loading="saving" />
          </div>
        </div>

        <!-- Appearance -->
        <div class="section">
          <div class="section-title">{{ t('profile.appearance.title') }}</div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.appearance.theme.title') }}</h3>
              <p>{{ t('profile.appearance.theme.description') }}</p>
            </div>
            <el-select v-model="appearance.theme" :disabled="saving" class="select-input">
              <el-option
                v-for="opt in themeOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.appearance.density.title') }}</h3>
              <p>{{ t('profile.appearance.density.description') }}</p>
            </div>
            <el-select v-model="appearance.messageDensity" :disabled="saving" class="select-input">
              <el-option
                v-for="opt in densityOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>

          <div class="pref-group">
            <div class="pref-label">
              <h3>{{ t('profile.appearance.language.title') }}</h3>
              <p>{{ t('profile.appearance.language.description') }}</p>
            </div>
            <el-select v-model="appearance.language" :disabled="saving" class="select-input">
              <el-option
                v-for="opt in languageOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>
        </div>

        <!-- Connected Accounts -->
        <div class="section">
          <div class="section-title">{{ t('profile.connectedAccounts.title') }}</div>

          <div class="connected-account">
            <div class="connected-account-icon">
              <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
              </svg>
            </div>
            <div class="connected-account-info">
              <div class="name">{{ t('profile.connectedAccounts.email') }}</div>
              <div class="detail">{{ currentUser?.email }}</div>
            </div>
            <span class="connected-status">● {{ t('profile.connectedAccounts.connected') }}</span>
          </div>
        </div>

        <!-- Danger Zone -->
        <div class="danger-zone">
          <h3>{{ t('profile.dangerZone.title') }}</h3>
          <p>{{ t('profile.dangerZone.description') }}</p>
          <div class="danger-actions">
            <el-button type="danger" plain @click="handleLogout">
              <el-icon><SwitchButton /></el-icon>
              {{ t('common.logout') }}
            </el-button>
            <el-button type="danger" plain @click="deleteAccount">
              <el-icon><Delete /></el-icon>
              {{ t('profile.dangerZone.deleteAccount') }}
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

.select-input {
  width: 160px;

  :deep(.el-input__wrapper) {
    background-color: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: none;

    &.is-focus {
      border-color: var(--accent);
    }
  }

  :deep(.el-input__inner) {
    color: var(--fg);
  }
}

// Connected Accounts
.connected-account {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 0;

  & + .connected-account {
    border-top: 1px solid var(--border);
  }
}

.connected-account-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  border: 1px solid var(--border);
  color: var(--muted);
  font-size: 20px;
}

.connected-account-info {
  flex: 1;

  .name {
    font-size: 14px;
    font-weight: 500;
  }

  .detail {
    font-size: 12px;
    color: var(--muted);
  }
}

.connected-status {
  font-size: 12px;
  color: var(--accent-green);
  display: flex;
  align-items: center;
  gap: 4px;
}

// Danger Zone
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
    margin-bottom: 12px;
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
    text-align: left;
  }

  .profile-meta {
    gap: 12px;
  }

  .pref-group {
    flex-direction: column;
    align-items: flex-start;
  }

  .select-input {
    width: 100%;
  }

}
</style>
