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
    enableFileUploadComplete: true,
    enableSound: true,
    enableDesktopNotification: true,
    enableDoNotDisturb: false,
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
    saveMediaGallery: false,
    imageQuality: 'high',
    autoPlayVideo: true,
    autoPlayAudio: false,
  },
}

/**
 * 用户设置状态管理 Store
 * 负责管理用户个性化设置的加载、更新和同步
 * 采用乐观更新策略：先更新本地状态，再异步同步到服务端
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
   * 乐观更新用户设置
   * 先更新本地状态实现即时反馈，再异步同步到服务端
   * @param newSettings 新的设置值（部分更新）
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateSettings(newSettings: PartialUserSettings): Promise<{ success: boolean; error?: string }> {
    // 保存旧值以便失败时回滚
    const oldSettings = { ...settings.value }

    // 乐观更新：立即更新本地状态
    settings.value = { ...settings.value, ...newSettings }

    // 异步同步到服务端
    try {
      const res = await settingsApi.updateSettings(newSettings)
      // 检查服务端返回的 success 字段
      if (res.success === false) {
        // 服务端返回业务错误
        let errorMsg: string | null = null
        // 1. 尝试获取 message 字段
        if (res.message) {
          errorMsg = res.message
        }
        // 2. 如果 error 是字符串，直接使用
        else if (typeof res.error === 'string') {
          errorMsg = res.error
        }
        // 3. 如果 error 是对象，转为 JSON 字符串
        else if (typeof res.error === 'object' && res.error !== null) {
          errorMsg = JSON.stringify(res.error)
        }
        settings.value = oldSettings
        return { success: false, error: errorMsg || '保存设置失败' }
      }
      if (res.data) {
        // 同步成功，使用服务端返回的数据（可能包含其他字段的更新）
        settings.value = { ...settings.value, ...res.data }
        return { success: true }
      }
      // 服务端返回成功但没有数据，保持乐观更新的状态
      // PATCH 请求通常只返回 200/204 而不返回数据
      return { success: true }
    } catch (err: unknown) {
      // 同步失败，回滚到旧值
      const error = err as {
        response?: { data?: { message?: string; error?: string | Record<string, unknown>; detail?: string }; status?: number; statusText?: string }
        message?: string
        code?: string
      }

      // 详细记录错误信息以便调试
      console.error('[SettingsStore] Failed to sync settings:', {
        error: err,
        response: error?.response,
        responseData: error?.response?.data,
        status: error?.response?.status,
        statusText: error?.response?.statusText,
        message: error?.message,
        code: error?.code,
      })

      // 提取错误消息（按优先级）
      let errorMsg: string | null = null

      // 1. 尝试获取 message 字段
      if (error?.response?.data?.message) {
        errorMsg = error.response.data.message
      }
      // 2. 如果 error 是字符串，直接使用
      else if (typeof error?.response?.data?.error === 'string') {
        errorMsg = error.response.data.error
      }
      // 3. 如果 error 是对象，尝试获取其中的 message 或转为 JSON
      else if (typeof error?.response?.data?.error === 'object' && error.response.data.error !== null) {
        const errObj = error.response.data.error as Record<string, unknown>
        errorMsg = (errObj.message as string) || JSON.stringify(errObj)
      }
      // 4. 其他字段
      else if (error?.response?.data?.detail) {
        errorMsg = error.response.data.detail
      }
      else if (error?.response?.statusText) {
        errorMsg = error.response.statusText
      }
      else if (error?.response?.status) {
        errorMsg = `HTTP ${error.response.status} 错误`
      }
      else if (error?.message) {
        errorMsg = error.message
      }

      settings.value = oldSettings
      return { success: false, error: errorMsg || '网络错误，请检查连接' }
    }
  }

  /**
   * 更新通知设置（乐观更新）
   * @param notification 新的通知设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateNotificationSettings(notification: Partial<NotificationSettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.notifications, ...notification }
    return updateSettings({ notifications: updated })
  }

  /**
   * 更新隐私设置（乐观更新）
   * @param privacy 新的隐私设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updatePrivacySettings(privacy: Partial<PrivacySettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.privacy, ...privacy }
    return updateSettings({ privacy: updated })
  }

  /**
   * 更新消息设置（乐观更新）
   * @param message 新的消息设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateMessageSettings(message: Partial<MessageSettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.message, ...message }
    return updateSettings({ message: updated })
  }

  /**
   * 更新安全设置（乐观更新）
   * @param security 新的安全设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateSecuritySettings(security: Partial<SecuritySettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.security, ...security }
    return updateSettings({ security: updated })
  }

  /**
   * 更新语言地区设置（乐观更新）
   * @param locale 新的语言地区设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateLocaleSettings(locale: Partial<LocaleSettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.locale, ...locale }
    return updateSettings({ locale: updated })
  }

  /**
   * 更新无障碍设置（乐观更新）
   * @param accessibility 新的无障碍设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateAccessibilitySettings(accessibility: Partial<AccessibilitySettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.accessibility, ...accessibility }
    return updateSettings({ accessibility: updated })
  }

  /**
   * 更新媒体设置（乐观更新）
   * @param media 新的媒体设置
   * @returns 成功返回 { success: true }，失败返回 { success: false, error: string }
   */
  async function updateMediaSettings(media: Partial<MediaSettings>): Promise<{ success: boolean; error?: string }> {
    const updated = { ...settings.value.media, ...media }
    return updateSettings({ media: updated })
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
