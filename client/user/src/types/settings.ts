/**
 * 用户设置类型定义
 * 对应后端 /api/v1/users/me/settings 接口
 * 所有字段使用 camelCase 命名规范
 */

// ========== 通知设置 ==========

/** 通知设置 */
export interface NotificationSettings {
  /** 总开关 - 是否启用通知 */
  enableNotification: boolean
  /** 私信通知 */
  enableDirectMessage: boolean
  /** @提及通知 */
  enableMention: boolean
  /** 房间邀请通知 */
  enableRoomInvitation: boolean
  /** 系统通知 */
  enableSystemNotification: boolean
  /** 声音提醒 */
  enableSound: boolean
  /** 桌面通知 */
  enableDesktopNotification: boolean
}

// ========== 隐私设置 ==========

/** 可见性选项 */
export type VisibilityOption = 'everyone' | 'friends' | 'none'

/** 隐私设置 */
export interface PrivacySettings {
  /** 在线状态可见性 */
  onlineStatusVisibility: VisibilityOption
  /** 个人资料可见性 */
  profileVisibility: VisibilityOption
  /** 允许陌生人私信 */
  allowStrangerMessage: boolean
  /** 允许房间邀请 */
  allowRoomInvitation: boolean
}

// ========== 消息设置 ==========

/** 消息设置 */
export interface MessageSettings {
  /** 消息预览 */
  showMessagePreview: boolean
  /** 已读回执 */
  enableReadReceipt: boolean
  /** 输入状态显示 */
  showTypingStatus: boolean
  /** 免打扰模式 */
  enableDoNotDisturb: boolean
}

// ========== 账号安全设置 ==========

/** 账号安全设置 */
export interface SecuritySettings {
  /** 异地登录提醒 */
  enableAbnormalLoginAlert: boolean
  /** 仅允许单设备登录 */
  enableSingleDeviceLogin: boolean
}

// ========== 语言与地区设置 ==========

/** 时间格式 */
export type TimeFormat = '12h' | '24h'

/** 日期格式 */
export type DateFormat = 'YYYY-MM-DD' | 'DD/MM/YYYY' | 'MM/DD/YYYY'

/** 星期起始日 */
export type WeekStartDay = 'monday' | 'sunday'

/** 语言与地区设置 */
export interface LocaleSettings {
  /** 界面语言 */
  language: string
  /** 时区 */
  timezone: string
  /** 时间格式 */
  timeFormat: TimeFormat
  /** 日期格式 */
  dateFormat: DateFormat
  /** 星期起始日 */
  weekStartDay: WeekStartDay
}

// ========== 无障碍设置 ==========

/** 字体大小 */
export type FontSize = 'small' | 'medium' | 'large'

/** 无障碍设置 */
export interface AccessibilitySettings {
  /** 字体大小 */
  fontSize: FontSize
  /** 减少动效 */
  reduceMotion: boolean
  /** 高对比度 */
  highContrast: boolean
  /** 紧凑模式 */
  compactMode: boolean
}

// ========== 媒体与存储设置 ==========

/** 图片质量 */
export type ImageQuality = 'original' | 'high' | 'medium' | 'low'

/** 媒体与存储设置 */
export interface MediaSettings {
  /** 自动下载媒体文件 */
  autoDownloadMedia: boolean
  /** 图片质量 */
  imageQuality: ImageQuality
  /** 自动播放视频 */
  autoPlayVideo: 'always' | 'wifi' | 'never'
}

// ========== 设备类型 ==========

/** 设备类型 */
export type DeviceType = 'mobile' | 'tablet' | 'desktop' | 'unknown'

/** 登录设备信息 */
export interface LoginDevice {
  /** 设备ID */
  deviceId: string
  /** 设备名称 */
  deviceName: string
  /** 设备类型 */
  deviceType: DeviceType
  /** IP地址 */
  ipAddress: string
  /** 位置 */
  location: string | null
  /** 最后活跃时间 */
  lastActiveAt: string
  /** 是否当前设备 */
  isCurrent: boolean
  /** 是否被禁用 */
  isBlocked: boolean
}

/** 登录历史记录 */
export interface LoginHistory {
  /** 记录ID */
  id: string
  /** 登录时间 */
  loginAt: string
  /** IP地址 */
  ipAddress: string
  /** 设备名称 */
  deviceName: string
  /** 设备类型 */
  deviceType: DeviceType
  /** 登录结果 */
  result: 'success' | 'failed'
  /** 风险等级 */
  riskLevel: 'low' | 'medium' | 'high'
}

// ========== 用户完整设置 ==========

/** 用户完整设置 */
export interface UserSettings {
  /** 通知设置 */
  notifications: NotificationSettings
  /** 隐私设置 */
  privacy: PrivacySettings
  /** 消息设置 */
  message: MessageSettings
  /** 账号安全设置 */
  security: SecuritySettings
  /** 语言与地区设置 */
  locale: LocaleSettings
  /** 无障碍设置 */
  accessibility: AccessibilitySettings
  /** 媒体与存储设置 */
  media: MediaSettings
}

/** 部分用户设置（用于更新） */
export type PartialUserSettings = Partial<UserSettings>

// ========== 房间级通知设置 ==========

/** 房间通知偏好 */
export type RoomNotificationPreference = 'all' | 'mention_only' | 'muted'

/** 房间级设置 */
export interface RoomUserSettings {
  /** 房间ID */
  roomId: string
  /** 房间名称 */
  roomName: string
  /** 通知偏好 */
  notificationPreference: RoomNotificationPreference
  /** 是否置顶 */
  isPinned: boolean
  /** 是否静音 */
  isMuted: boolean
}
