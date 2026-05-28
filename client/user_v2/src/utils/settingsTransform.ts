/**
 * 设置字段转换工具
 * 处理前端 camelCase 与后端 snake_case 之间的字段名转换
 */

import type {
  UserSettings,
  PartialUserSettings,
  NotificationSettings,
  PrivacySettings,
  MessageSettings,
  LocaleSettings,
  AccessibilitySettings,
  MediaSettings,
} from '@/types/settings'

// ========== 后端字段类型定义 (snake_case) ==========

interface BackendNotificationSettings {
  private_message: boolean
  mentioned: boolean
  room_invitation: boolean
  system_notification: boolean
  file_upload_complete: boolean
  sound_enabled: boolean
  desktop_notification: boolean
  do_not_disturb: boolean
}

interface BackendPrivacySettings {
  online_status_visibility: 'everyone' | 'friends' | 'nobody'
  profile_visibility: 'everyone' | 'friends' | 'nobody'
  allow_stranger_message: boolean
  allow_room_invitation: boolean
  single_device_login: boolean
}

interface BackendMessageSettings {
  message_preview: boolean
  read_receipt: boolean
  typing_indicator: boolean
  do_not_disturb: boolean
}

interface BackendLanguageSettings {
  language: string
  timezone: string
  time_format: '12h' | '24h'
  date_format: 'YYYY-MM-DD' | 'DD/MM/YYYY' | 'MM/DD/YYYY'
  first_day_of_week: 'monday' | 'sunday'
}

interface BackendAccessibilitySettings {
  font_size: 'small' | 'medium' | 'large'
  reduce_motion: boolean
  high_contrast: boolean
  dense_mode: boolean
}

interface BackendMediaSettings {
  auto_download_media: boolean
  save_media_gallery: boolean
  image_quality: 'original' | 'high' | 'medium' | 'low'
  auto_play_video: boolean
  auto_play_audio: boolean
}

interface BackendUserSettings {
  notification?: BackendNotificationSettings
  privacy?: BackendPrivacySettings
  message?: BackendMessageSettings
  language?: BackendLanguageSettings
  accessibility?: BackendAccessibilitySettings
  media?: BackendMediaSettings
}

// ========== 转换函数 ==========

/**
 * 将前端的 VisibilityOption 转换为后端的 Visibility
 */
function toBackendVisibility(visibility: 'everyone' | 'friends' | 'none'): 'everyone' | 'friends' | 'nobody' {
  return visibility === 'none' ? 'nobody' : visibility
}

/**
 * 将后端的 Visibility 转换为前端的 VisibilityOption
 */
function fromBackendVisibility(visibility: 'everyone' | 'friends' | 'nobody'): 'everyone' | 'friends' | 'none' {
  return visibility === 'nobody' ? 'none' : visibility
}

/**
 * 通知设置：前端 -> 后端
 */
function toBackendNotification(frontend: NotificationSettings): BackendNotificationSettings {
  return {
    private_message: frontend.enableDirectMessage,
    mentioned: frontend.enableMention,
    room_invitation: frontend.enableRoomInvitation,
    system_notification: frontend.enableSystemNotification,
    file_upload_complete: frontend.enableFileUploadComplete,
    sound_enabled: frontend.enableSound,
    desktop_notification: frontend.enableDesktopNotification,
    do_not_disturb: frontend.enableDoNotDisturb,
  }
}

/**
 * 通知设置：后端 -> 前端
 */
function fromBackendNotification(backend: BackendNotificationSettings): NotificationSettings {
  return {
    enableNotification: true, // 总开关，后端没有对应字段，默认true
    enableDirectMessage: backend.private_message,
    enableMention: backend.mentioned,
    enableRoomInvitation: backend.room_invitation,
    enableSystemNotification: backend.system_notification,
    enableFileUploadComplete: backend.file_upload_complete,
    enableSound: backend.sound_enabled,
    enableDesktopNotification: backend.desktop_notification,
    enableDoNotDisturb: backend.do_not_disturb,
  }
}

/**
 * 隐私设置：前端 -> 后端
 */
function toBackendPrivacy(frontend: PrivacySettings): BackendPrivacySettings {
  return {
    online_status_visibility: toBackendVisibility(frontend.onlineStatusVisibility),
    profile_visibility: toBackendVisibility(frontend.profileVisibility),
    allow_stranger_message: frontend.allowStrangerMessage,
    allow_room_invitation: frontend.allowRoomInvitation,
    single_device_login: false, // 安全设置中的字段，这里默认false
  }
}

/**
 * 隐私设置：后端 -> 前端
 */
function fromBackendPrivacy(backend: BackendPrivacySettings): PrivacySettings {
  return {
    onlineStatusVisibility: fromBackendVisibility(backend.online_status_visibility),
    profileVisibility: fromBackendVisibility(backend.profile_visibility),
    allowStrangerMessage: backend.allow_stranger_message,
    allowRoomInvitation: backend.allow_room_invitation,
  }
}

/**
 * 消息设置：前端 -> 后端
 */
function toBackendMessage(frontend: MessageSettings): BackendMessageSettings {
  return {
    message_preview: frontend.showMessagePreview,
    read_receipt: frontend.enableReadReceipt,
    typing_indicator: frontend.showTypingStatus,
    do_not_disturb: frontend.enableDoNotDisturb,
  }
}

/**
 * 消息设置：后端 -> 前端
 */
function fromBackendMessage(backend: BackendMessageSettings): MessageSettings {
  return {
    showMessagePreview: backend.message_preview,
    enableReadReceipt: backend.read_receipt,
    showTypingStatus: backend.typing_indicator,
    enableDoNotDisturb: backend.do_not_disturb,
  }
}

/**
 * 语言设置：前端 -> 后端
 */
function toBackendLanguage(frontend: LocaleSettings): BackendLanguageSettings {
  return {
    language: frontend.language,
    timezone: frontend.timezone,
    time_format: frontend.timeFormat,
    date_format: frontend.dateFormat,
    first_day_of_week: frontend.weekStartDay,
  }
}

/**
 * 语言设置：后端 -> 前端
 */
function fromBackendLanguage(backend: BackendLanguageSettings): LocaleSettings {
  return {
    language: backend.language,
    timezone: backend.timezone,
    timeFormat: backend.time_format,
    dateFormat: backend.date_format,
    weekStartDay: backend.first_day_of_week,
  }
}

/**
 * 无障碍设置：前端 -> 后端
 */
function toBackendAccessibility(frontend: AccessibilitySettings): BackendAccessibilitySettings {
  return {
    font_size: frontend.fontSize,
    reduce_motion: frontend.reduceMotion,
    high_contrast: frontend.highContrast,
    dense_mode: frontend.compactMode,
  }
}

/**
 * 无障碍设置：后端 -> 前端
 */
function fromBackendAccessibility(backend: BackendAccessibilitySettings): AccessibilitySettings {
  return {
    fontSize: backend.font_size,
    reduceMotion: backend.reduce_motion,
    highContrast: backend.high_contrast,
    compactMode: backend.dense_mode,
  }
}

/**
 * 媒体设置：前端 -> 后端
 */
function toBackendMedia(frontend: MediaSettings): BackendMediaSettings {
  return {
    auto_download_media: frontend.autoDownloadMedia,
    save_media_gallery: frontend.saveMediaGallery,
    image_quality: frontend.imageQuality,
    auto_play_video: frontend.autoPlayVideo,
    auto_play_audio: frontend.autoPlayAudio,
  }
}

/**
 * 媒体设置：后端 -> 前端
 */
function fromBackendMedia(backend: BackendMediaSettings): MediaSettings {
  return {
    autoDownloadMedia: backend.auto_download_media,
    saveMediaGallery: backend.save_media_gallery,
    imageQuality: backend.image_quality,
    autoPlayVideo: backend.auto_play_video,
    autoPlayAudio: backend.auto_play_audio,
  }
}

// ========== 导出转换函数 ==========

/**
 * 将前端用户设置转换为后端格式（用于请求）
 */
export function toBackendSettings(frontend: PartialUserSettings): Partial<BackendUserSettings> {
  const result: Partial<BackendUserSettings> = {}

  if (frontend.notifications) {
    result.notification = toBackendNotification(frontend.notifications)
  }
  if (frontend.privacy) {
    result.privacy = toBackendPrivacy(frontend.privacy)
  }
  if (frontend.message) {
    result.message = toBackendMessage(frontend.message)
  }
  if (frontend.locale) {
    result.language = toBackendLanguage(frontend.locale)
  }
  if (frontend.accessibility) {
    result.accessibility = toBackendAccessibility(frontend.accessibility)
  }
  if (frontend.media) {
    result.media = toBackendMedia(frontend.media)
  }

  return result
}

/**
 * 将后端用户设置转换为前端格式（用于响应）
 */
export function fromBackendSettings(backend: BackendUserSettings): Partial<UserSettings> {
  const result: Partial<UserSettings> = {}

  if (backend.notification) {
    result.notifications = fromBackendNotification(backend.notification)
  }
  if (backend.privacy) {
    result.privacy = fromBackendPrivacy(backend.privacy)
  }
  if (backend.message) {
    result.message = fromBackendMessage(backend.message)
  }
  if (backend.language) {
    result.locale = fromBackendLanguage(backend.language)
  }
  if (backend.accessibility) {
    result.accessibility = fromBackendAccessibility(backend.accessibility)
  }
  if (backend.media) {
    result.media = fromBackendMedia(backend.media)
  }

  return result
}

export type { BackendUserSettings }
