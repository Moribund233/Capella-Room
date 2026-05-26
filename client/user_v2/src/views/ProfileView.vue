<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowLeft,
  Clock,
  User,
  Star,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()

// 用户设置
const settings = ref({
  notifications: true,
  messagePreviews: true,
  soundEffects: false,
  onlineStatus: true,
})

// 外观设置
const appearance = ref({
  theme: 'Dark',
  messageDensity: 'Comfortable',
  language: 'English',
})

// 连接账户
const connectedAccounts = [
  { name: 'Google', email: 'alex@example.com', icon: 'google', connected: true },
  { name: 'GitHub', email: 'alex-dev', icon: 'github', connected: true },
]

/**
 * 返回应用
 */
function goBack() {
  router.push('/app')
}

/**
 * 断开账户连接
 * @param account - 账户信息
 */
function disconnectAccount(account: typeof connectedAccounts[0]) {
  // TODO: 实现断开连接逻辑
  ElMessage.info(`Disconnecting ${account.name}...`)
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
      confirmButtonClass: 'el-button--danger',
    }
  ).then(() => {
    ElMessage.success(t('common.success'))
    router.push('/')
  }).catch(() => {
    // 取消删除
  })
}
</script>

<template>
  <div class="profile-page">
    <!-- 返回栏 -->
    <div class="back-bar">
      <a href="#" @click.prevent="goBack">
        <el-icon><ArrowLeft /></el-icon>
        {{ t('profile.backToWave') }}
      </a>
    </div>

    <div class="container">
      <!-- 个人资料头部 -->
      <div class="profile-header">
        <div class="profile-avatar">
          <span>A</span>
          <span class="status-big"></span>
        </div>
        <div class="profile-info">
          <h1>alex</h1>
          <p class="handle">@alex · alex@example.com</p>
          <p class="bio">
            Design lead at Wave. Building the future of community conversations.
            Previously product design at Linear. Passionate about design systems, dark mode, and good coffee.
          </p>
          <div class="profile-meta">
            <span>
              <el-icon><Clock /></el-icon>
              {{ t('profile.joined') }} March 2025
            </span>
            <span>
              <el-icon><User /></el-icon>
              8 {{ t('profile.mutualServers') }}
            </span>
            <span>
              <el-icon><Star /></el-icon>
              {{ t('profile.admin') }}
            </span>
          </div>
          <div class="profile-actions">
            <el-button type="primary">{{ t('profile.message') }}</el-button>
            <el-button>{{ t('profile.shareProfile') }}</el-button>
          </div>
        </div>
      </div>

      <!-- 偏好设置 -->
      <div class="section">
        <div class="section-title">{{ t('profile.preferences.title') }}</div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.preferences.notifications.title') }}</h3>
            <p>{{ t('profile.preferences.notifications.description') }}</p>
          </div>
          <el-switch v-model="settings.notifications" />
        </div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.preferences.messagePreviews.title') }}</h3>
            <p>{{ t('profile.preferences.messagePreviews.description') }}</p>
          </div>
          <el-switch v-model="settings.messagePreviews" />
        </div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.preferences.soundEffects.title') }}</h3>
            <p>{{ t('profile.preferences.soundEffects.description') }}</p>
          </div>
          <el-switch v-model="settings.soundEffects" />
        </div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.preferences.onlineStatus.title') }}</h3>
            <p>{{ t('profile.preferences.onlineStatus.description') }}</p>
          </div>
          <el-switch v-model="settings.onlineStatus" />
        </div>
      </div>

      <!-- 外观设置 -->
      <div class="section">
        <div class="section-title">{{ t('profile.appearance.title') }}</div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.appearance.theme.title') }}</h3>
            <p>{{ t('profile.appearance.theme.description') }}</p>
          </div>
          <el-select v-model="appearance.theme" style="width: 160px;">
            <el-option :label="t('profile.appearance.theme.dark')" value="Dark" />
            <el-option :label="t('profile.appearance.theme.light')" value="Light" />
            <el-option :label="t('profile.appearance.theme.system')" value="System" />
          </el-select>
        </div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.appearance.messageDensity.title') }}</h3>
            <p>{{ t('profile.appearance.messageDensity.description') }}</p>
          </div>
          <el-select v-model="appearance.messageDensity" style="width: 160px;">
            <el-option :label="t('profile.appearance.messageDensity.comfortable')" value="Comfortable" />
            <el-option :label="t('profile.appearance.messageDensity.compact')" value="Compact" />
          </el-select>
        </div>

        <div class="pref-group">
          <div class="pref-label">
            <h3>{{ t('profile.appearance.language.title') }}</h3>
            <p>{{ t('profile.appearance.language.description') }}</p>
          </div>
          <el-select v-model="appearance.language" style="width: 160px;">
            <el-option label="English" value="English" />
            <el-option label="简体中文" value="Chinese" />
            <el-option label="日本語" value="Japanese" />
          </el-select>
        </div>
      </div>

      <!-- 连接账户 -->
      <div class="section">
        <div class="section-title">{{ t('profile.connectedAccounts.title') }}</div>

        <div
          v-for="account in connectedAccounts"
          :key="account.name"
          class="connected-account"
        >
          <div class="connected-account-icon">
            <svg v-if="account.icon === 'google'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="14" rx="2"/>
              <path d="M22 5l-10 7L2 5"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M16 8a6 6 0 016-6v6a6 6 0 01-6 6"/>
              <path d="M2 16a6 6 0 016-6v6a6 6 0 01-6 6"/>
              <circle cx="8" cy="8" r="2"/>
              <circle cx="8" cy="16" r="2"/>
              <circle cx="16" cy="8" r="2"/>
            </svg>
          </div>
          <div class="connected-account-info">
            <div class="name">{{ account.name }}</div>
            <div class="detail">{{ account.email }}</div>
          </div>
          <span class="connected-status">● {{ t('profile.connectedAccounts.connected') }}</span>
          <el-button text @click="disconnectAccount(account)">{{ t('profile.connectedAccounts.disconnect') }}</el-button>
        </div>
      </div>

      <!-- 危险区域 -->
      <div class="danger-zone">
        <h3>{{ t('profile.dangerZone.title') }}</h3>
        <p>{{ t('profile.dangerZone.description') }}</p>
        <el-button type="danger" plain @click="deleteAccount">{{ t('profile.dangerZone.deleteAccount') }}</el-button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.profile-page {
  background: var(--wave-bg);
  color: var(--wave-fg);
  min-height: 100vh;
  padding-top: 60px;
}

.container {
  max-width: 720px;
  margin-inline: auto;
  padding-inline: 24px;
}

.back-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: color-mix(in oklch, var(--wave-bg) 95%, transparent);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--wave-border);

  a {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--wave-muted);
    font-size: 14px;
    text-decoration: none;

    &:hover {
      color: var(--wave-fg);
    }
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
  background: linear-gradient(135deg, var(--wave-accent), var(--wave-accent-pink));
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
  background: var(--wave-accent-green);
  border: 3px solid var(--wave-bg);
}

.profile-info {
  flex: 1;

  h1 {
    font-family: var(--wave-font-display);
    font-size: 28px;
    font-weight: 600;
    margin-bottom: 4px;
  }
}

.handle {
  font-size: 15px;
  color: var(--wave-muted);
  margin-bottom: 8px;
}

.bio {
  font-size: 14px;
  color: var(--wave-muted);
  max-width: 48ch;
  line-height: 1.5;
}

.profile-meta {
  display: flex;
  gap: 24px;
  margin-top: 12px;

  span {
    font-size: 13px;
    color: var(--wave-muted);
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
  border-top: 1px solid var(--wave-border);
  padding: 24px 0;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--wave-muted);
  padding-bottom: 16px;
}

// 设置项
.pref-group {
  padding: 16px 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;

  & + .pref-group {
    border-top: 1px solid var(--wave-border);
  }
}

.pref-label {
  h3 {
    font-size: 15px;
    font-weight: 500;
    margin: 0;
  }

  p {
    font-size: 13px;
    color: var(--wave-muted);
    margin-top: 2px;
  }
}

:deep(.el-switch__core) {
  background-color: var(--wave-border);
  border-color: var(--wave-border);
}

:deep(.el-switch.is-checked .el-switch__core) {
  background-color: var(--wave-accent);
  border-color: var(--wave-accent);
}

:deep(.el-select .el-input__wrapper) {
  background-color: var(--wave-surface);
}

// 连接账户
.connected-account {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 0;

  & + .connected-account {
    border-top: 1px solid var(--wave-border);
  }
}

.connected-account-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--wave-radius);
  display: grid;
  place-items: center;
  border: 1px solid var(--wave-border);
  color: var(--wave-muted);

  svg {
    width: 20px;
    height: 20px;
  }
}

.connected-account-info {
  flex: 1;

  .name {
    font-size: 14px;
    font-weight: 500;
  }

  .detail {
    font-size: 12px;
    color: var(--wave-muted);
  }
}

.connected-status {
  font-size: 12px;
  color: var(--wave-accent-green);
  display: flex;
  align-items: center;
  gap: 4px;
}

// 危险区域
.danger-zone {
  padding: 20px;
  border: 1px solid color-mix(in oklch, var(--wave-accent-orange) 40%, transparent);
  border-radius: var(--wave-radius-lg);
  margin: 24px 0 48px;

  h3 {
    font-size: 15px;
    font-weight: 600;
    color: var(--wave-accent-orange);
    margin-bottom: 4px;
  }

  p {
    font-size: 13px;
    color: var(--wave-muted);
    margin-bottom: 12px;
  }
}

@media (max-width: 640px) {
  .profile-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .profile-meta {
    flex-wrap: wrap;
    gap: 12px;
  }

  .pref-group {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
