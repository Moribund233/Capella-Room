import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { settingsApi, securityApi } from '@/api/settings'
import type {
  UserSettings,
  PartialUserSettings,
  NotificationSettings,
  PrivacySettings,
  MessageSettings,
  SecuritySettings,
  LocaleSettings,
  AccessibilitySettings,
  MediaSettings,
  LoginDevice,
  LoginHistory,
} from '@/types/settings'

/** 默认用户设置 */
const DEFAULT_SETTINGS: UserSettings = {
  notifications: {
    enableNotification: true,
    enableDirectMessage: true,
    enableMention: true,
    enableRoomInvitation: true,
    enableSystemNotification: true,
    enableSound: true,
    enableDesktopNotification: true,
  },
  privacy: {
    onlineStatusVisibility: 'everyone',
    profileVisibility: 'everyone',
    allowStrangerMessage: true,
    allowRoomInvitation: true,
  },
  message: {
    showMessagePreview: true,
    enableReadReceipt: true,
    showTypingStatus: true,
    enableDoNotDisturb: false,
  },
  security: {
    enableAbnormalLoginAlert: true,
    enableSingleDeviceLogin: false,
  },
  locale: {
    language: 'zh-CN',
    timezone: 'Asia/Shanghai',
    timeFormat: '24h',
    dateFormat: 'YYYY-MM-DD',
    weekStartDay: 'monday',
  },
  accessibility: {
    fontSize: 'medium',
    reduceMotion: false,
    highContrast: false,
    compactMode: false,
  },
  media: {
    autoDownloadMedia: true,
    imageQuality: 'high',
    autoPlayVideo: 'wifi',
  },
}

/**
 * 用户设置状态管理 Store
 * 负责管理用户个性化设置的加载、更新和同步
 */
export const useSettingsStore = defineStore('settings', () => {
  // ========== State ==========
  /** 用户设置 */
  const settings = ref<UserSettings>({ ...DEFAULT_SETTINGS })
  /** 是否已加载 */
  const loaded = ref(false)
  /** 是否正在加载 */
  const loading = ref(false)
  /** 是否正在保存 */
  const saving = ref(false)
  /** 加载错误信息 */
  const error = ref<string | null>(null)

  /** 登录设备列表 */
  const loginDevices = ref<LoginDevice[]>([])
  /** 登录历史记录 */
  const loginHistory = ref<LoginHistory[]>([])
  /** 安全相关加载状态 */
  const securityLoading = ref(false)

  // ========== Getters ==========
  /** 通知设置 */
  const notificationSettings = computed(() => settings.value.notifications)
  /** 隐私设置 */
  const privacySettings = computed(() => settings.value.privacy)
  /** 消息设置 */
  const messageSettings = computed(() => settings.value.message)
  /** 安全设置 */
  const securitySettings = computed(() => settings.value.security)
  /** 语言地区设置 */
  const localeSettings = computed(() => settings.value.locale)
  /** 无障碍设置 */
  const accessibilitySettings = computed(() => settings.value.accessibility)
  /** 媒体设置 */
  const mediaSettings = computed(() => settings.value.media)

  // ========== Actions ==========

  /**
   * 加载用户设置
   * 从服务器获取用户设置，失败时使用默认设置
   */
  async function loadSettings(): Promise<boolean> {
    if (loading.value) return false

    loading.value = true
    error.value = null

    try {
      const res = await settingsApi.getSettings()
      if (res.data) {
        settings.value = { ...DEFAULT_SETTINGS, ...res.data }
        loaded.value = true
        return true
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : '加载设置失败'
      console.error('[SettingsStore] Failed to load settings:', err)
    } finally {
      loading.value = false
    }

    return false
  }

  /**
   * 确保设置已加载
   * 如果未加载则自动加载
   */
  async function ensureLoaded(): Promise<boolean> {
    if (loaded.value) return true
    return loadSettings()
  }

  /**
   * 更新用户设置
   * @param newSettings 新的设置值（部分更新）
   */
  async function updateSettings(newSettings: PartialUserSettings): Promise<boolean> {
    saving.value = true

    try {
      const res = await settingsApi.updateSettings(newSettings)
      if (res.data) {
        settings.value = { ...settings.value, ...res.data }
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to update settings:', err)
    } finally {
      saving.value = false
    }

    return false
  }

  /**
   * 更新通知设置
   */
  async function updateNotificationSettings(notification: NotificationSettings): Promise<boolean> {
    return updateSettings({ notifications: notification })
  }

  /**
   * 更新隐私设置
   */
  async function updatePrivacySettings(privacy: PrivacySettings): Promise<boolean> {
    return updateSettings({ privacy })
  }

  /**
   * 更新消息设置
   */
  async function updateMessageSettings(message: MessageSettings): Promise<boolean> {
    return updateSettings({ message })
  }

  /**
   * 更新安全设置
   */
  async function updateSecuritySettings(security: SecuritySettings): Promise<boolean> {
    return updateSettings({ security })
  }

  /**
   * 更新语言地区设置
   */
  async function updateLocaleSettings(locale: LocaleSettings): Promise<boolean> {
    return updateSettings({ locale })
  }

  /**
   * 更新无障碍设置
   */
  async function updateAccessibilitySettings(accessibility: AccessibilitySettings): Promise<boolean> {
    return updateSettings({ accessibility })
  }

  /**
   * 更新媒体设置
   */
  async function updateMediaSettings(media: MediaSettings): Promise<boolean> {
    return updateSettings({ media })
  }

  /**
   * 重置设置为默认值
   */
  async function resetSettings(): Promise<boolean> {
    saving.value = true

    try {
      const res = await settingsApi.resetSettings()
      if (res.data) {
        settings.value = res.data
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to reset settings:', err)
    } finally {
      saving.value = false
    }

    return false
  }

  // ========== 安全相关 Actions ==========

  /**
   * 加载登录设备列表
   */
  async function loadLoginDevices(): Promise<boolean> {
    securityLoading.value = true

    try {
      const res = await securityApi.getLoginDevices()
      if (res.data?.devices) {
        loginDevices.value = res.data.devices
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to load devices:', err)
    } finally {
      securityLoading.value = false
    }

    return false
  }

  /**
   * 登出指定设备
   */
  async function logoutDevice(deviceId: string): Promise<boolean> {
    try {
      const res = await securityApi.logoutDevice(deviceId)
      if (res.success) {
        loginDevices.value = loginDevices.value.filter(d => d.deviceId !== deviceId)
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to logout device:', err)
    }

    return false
  }

  /**
   * 禁用指定设备
   */
  async function blockDevice(deviceId: string): Promise<boolean> {
    try {
      const res = await securityApi.blockDevice(deviceId)
      if (res.success) {
        const device = loginDevices.value.find(d => d.deviceId === deviceId)
        if (device) {
          device.isBlocked = true
        }
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to block device:', err)
    }

    return false
  }

  /**
   * 启用被禁用的设备
   */
  async function unblockDevice(deviceId: string): Promise<boolean> {
    try {
      const res = await securityApi.unblockDevice(deviceId)
      if (res.success) {
        const device = loginDevices.value.find(d => d.deviceId === deviceId)
        if (device) {
          device.isBlocked = false
        }
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to unblock device:', err)
    }

    return false
  }

  /**
   * 加载登录历史
   * @param params 查询参数（limit/offset）
   */
  async function loadLoginHistory(params?: { limit?: number; offset?: number }): Promise<boolean> {
    securityLoading.value = true

    try {
      const res = await securityApi.getLoginHistory(params)
      if (res.data?.history) {
        loginHistory.value = res.data.history
        return true
      }
    } catch (err) {
      console.error('[SettingsStore] Failed to load login history:', err)
    } finally {
      securityLoading.value = false
    }

    return false
  }

  return {
    // State
    settings,
    loaded,
    loading,
    saving,
    error,
    loginDevices,
    loginHistory,
    securityLoading,
    // Getters
    notificationSettings,
    privacySettings,
    messageSettings,
    securitySettings,
    localeSettings,
    accessibilitySettings,
    mediaSettings,
    // Actions
    loadSettings,
    ensureLoaded,
    updateSettings,
    updateNotificationSettings,
    updatePrivacySettings,
    updateMessageSettings,
    updateSecuritySettings,
    updateLocaleSettings,
    updateAccessibilitySettings,
    updateMediaSettings,
    resetSettings,
    loadLoginDevices,
    logoutDevice,
    blockDevice,
    unblockDevice,
    loadLoginHistory,
  }
})
