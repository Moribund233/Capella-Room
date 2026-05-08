import type { MessageApi } from 'naive-ui'

/**
 * 扩展 Window 接口以支持 Naive UI 的全局属性
 */
declare global {
  interface Window {
    $message?: MessageApi
  }
}

// ==================== 基础类型 ====================

/**
 * 响应式断点类型
 */
export type Breakpoint = 'mobile' | 'tablet' | 'desktop'

/**
 * 主题类型
 */
export type Theme = 'light' | 'dark'

/**
 * 用户角色类型
 */
export type UserRole = 'user' | 'admin' | 'super_admin'

/**
 * 用户在线状态类型
 */
export type UserStatus = 'online' | 'offline' | 'away'

/**
 * 用户账号状态类型
 */
export type UserAccountStatus = 'active' | 'disabled'

// ==================== 菜单相关类型 ====================

/**
 * 菜单项类型
 */
export interface MenuItem {
  /** 唯一标识 */
  key: string
  /** 显示文本 */
  label: string
  /** 图标组件 */
  icon?: unknown
  /** 路由路径 */
  path?: string
  /** 子菜单 */
  children?: MenuItem[]
  /** 是否禁用 */
  disabled?: boolean
}

// ==================== 用户相关类型 ====================

/**
 * 用户信息类型
 */
export interface UserInfo {
  /** 用户ID */
  id: string
  /** 用户名 */
  username: string
  /** 昵称 */
  nickname?: string
  /** 头像URL */
  avatar_url?: string | null
  /** 邮箱 */
  email?: string
  /** 在线状态：online/offline/away */
  status?: UserStatus
  /** 账号状态：true=启用, false=禁用 */
  is_active?: boolean
  /** 用户角色 */
  role?: UserRole
  /** 创建时间 */
  created_at?: string
}

/**
 * 登录参数
 */
export interface LoginParams {
  /** 邮箱 */
  email: string
  /** 密码 */
  password: string
}

/**
 * 登录结果
 */
export interface LoginResult {
  /** 访问令牌 */
  access_token: string
  /** 刷新令牌 */
  refresh_token: string
  /** 令牌类型 */
  token_type: string
  /** 过期时间（秒） */
  expires_in: number
  /** 用户信息 */
  user: UserInfo
}

/**
 * 注册参数
 */
export interface RegisterParams {
  /** 用户名 */
  username: string
  /** 邮箱 */
  email: string
  /** 密码 */
  password: string
}

/**
 * 外观配置
 * 用户可自定义的应用外观设置
 */
export interface AppearanceConfig {
  /** 背景图片 URL */
  backgroundImage: string
  /** 背景图片不透明度 (0-1) */
  backgroundOpacity: number
  /** 强调色（hex 格式），null 表示使用主题默认色 */
  accentColor: string | null
}

// ==================== 房间相关类型 ====================

/**
 * 房间类型
 */
export interface Room {
  /** 房间ID */
  id: string
  /** 房间名称 */
  name: string
  /** 房间描述 */
  description?: string | null
  /** 房间类型：public/private */
  room_type: 'public' | 'private'
  /** 成员数量 */
  member_count: number
  /** 最大成员数 */
  max_members: number
  /** 创建者ID */
  created_by: string
  /** 创建者名称 */
  created_by_name?: string
  /** 创建时间 */
  created_at: string
  /** 更新时间 */
  updated_at: string
  /** 是否已加入 */
  is_joined?: boolean
  /** 当前用户角色 */
  user_role?: 'owner' | 'admin' | 'member'
}

/**
 * 房间成员类型
 */
export interface RoomMember {
  /** 用户ID */
  id: string
  /** 用户名 */
  username: string
  /** 昵称 */
  nickname?: string
  /** 头像URL */
  avatar_url?: string | null
  /** 角色 */
  role: 'owner' | 'admin' | 'member'
  /** 加入时间 */
  joined_at: string
}

// ==================== 消息相关类型 ====================

/**
 * 消息类型
 */
export type MessageType = 'text' | 'image' | 'file' | 'system'

/**
 * 消息类型
 */
export interface Message {
  /** 消息ID */
  id: string
  /** 房间ID */
  room_id: string
  /** 发送者ID */
  sender_id: string
  /** 发送者名称 */
  sender_name: string
  /** 发送者头像 */
  sender_avatar?: string | null
  /** 消息类型 */
  message_type: MessageType
  /** 消息内容 */
  content: string
  /** 回复的消息ID */
  reply_to?: string | null
  /** 回复的消息信息 */
  reply_to_message?: ReplyToInfo | null
  /** 创建时间 */
  created_at: string
  /** 更新时间 */
  updated_at?: string | null
  /** 是否已删除 */
  is_deleted?: boolean
}

/**
 * 被引用消息的信息
 */
export interface ReplyToInfo {
  /** 消息ID */
  id: string
  /** 发送者ID */
  sender_id: string
  /** 发送者名称 */
  sender_name: string
  /** 内容 */
  content: string
  /** 创建时间 */
  created_at: string
}

/**
 * 发送消息参数
 */
export interface SendMessageParams {
  /** 房间ID */
  room_id: string
  /** 消息内容 */
  content: string
  /** 回复的消息ID */
  reply_to?: string
}

// ==================== 文件相关类型 ====================

/**
 * 文件类型
 */
export interface FileItem {
  /** 文件ID */
  id: string
  /** 文件名 */
  file_name: string
  /** 文件URL */
  file_url: string
  /** 文件大小 */
  file_size: number
  /** 文件类型 */
  file_type: string
  /** MIME类型 */
  mime_type: string
  /** 上传者ID */
  uploaded_by: string
  /** 上传者名称 */
  uploaded_by_name?: string
  /** 上传时间 */
  uploaded_at: string
}

// ==================== 审计日志相关类型 ====================

/**
 * 审计日志事件类型
 */
export type AuditEventType =
  | 'user_login'
  | 'user_logout'
  | 'user_register'
  | 'user_update'
  | 'user_delete'
  | 'room_create'
  | 'room_update'
  | 'room_delete'
  | 'room_join'
  | 'room_leave'
  | 'message_send'
  | 'message_delete'
  | 'message_edit'
  | 'file_upload'
  | 'file_delete'
  | 'config_change'
  | 'security_alert'
  | 'permission_change'
  | 'admin_action'
  | 'system_event'

/**
 * 审计日志严重程度
 */
export type AuditSeverity = 'info' | 'warning' | 'error' | 'critical'

/**
 * 审计日志项
 */
export interface AuditLogItem {
  /** 日志ID */
  id: string
  /** 事件类型 */
  event_type: AuditEventType
  /** 严重程度 */
  severity: AuditSeverity
  /** 操作者ID */
  actor_id: string
  /** 操作者类型 */
  actor_type: 'user' | 'system' | 'anonymous'
  /** 操作者名称 */
  actor_name?: string
  /** 操作者IP */
  actor_ip?: string
  /** 目标类型 */
  target_type?: string
  /** 目标ID */
  target_id?: string
  /** 操作描述 */
  action: string
  /** 请求详情 */
  request_details?: Record<string, unknown>
  /** 响应状态 */
  response_status?: number
  /** 创建时间 */
  created_at: string
}

/**
 * 安全告警项
 */
export interface SecurityAlert {
  /** 告警ID */
  id: string
  /** 告警类型 */
  alert_type: string
  /** 严重程度 */
  severity: AuditSeverity
  /** 告警标题 */
  title: string
  /** 告警描述 */
  description: string
  /** 来源IP */
  source_ip?: string
  /** 相关用户ID */
  user_id?: string
  /** 相关用户名 */
  username?: string
  /** 状态 */
  status: 'new' | 'acknowledged' | 'resolved' | 'ignored'
  /** 创建时间 */
  created_at: string
  /** 更新时间 */
  updated_at?: string
}

/**
 * 告警规则
 */
export interface AlertRule {
  /** 规则ID */
  id: string
  /** 规则名称 */
  name: string
  /** 事件类型 */
  event_type: AuditEventType
  /** 严重程度 */
  severity: AuditSeverity
  /** 条件描述 */
  condition: string
  /** 是否启用 */
  is_enabled: boolean
  /** 通知管理员 */
  notify_admins: boolean
  /** 创建时间 */
  created_at: string
}

// ==================== 统计相关类型 ====================

/**
 * 系统统计信息
 */
export interface SystemStats {
  /** 总用户数 */
  total_users: number
  /** 在线用户数 */
  online_users: number
  /** 总房间数 */
  total_rooms: number
  /** 活跃房间数 */
  active_rooms: number
  /** 今日消息数 */
  today_messages: number
  /** 总消息数 */
  total_messages: number
  /** 系统运行时间（秒） */
  uptime_seconds: number
  /** 数据库连接状态 */
  database_status: 'connected' | 'disconnected'
  /** Redis连接状态 */
  redis_status: 'connected' | 'disconnected'
}

/**
 * 活跃度统计
 */
export interface ActivityStats {
  /** 日期 */
  date: string
  /** 活跃用户数 */
  active_users: number
  /** 新用户数 */
  new_users: number
  /** 消息数 */
  message_count: number
}

/**
 * 性能指标
 */
export interface PerformanceMetrics {
  /** CPU使用率 */
  cpu_usage: number
  /** 内存使用率 */
  memory_usage: number
  /** 内存使用（MB） */
  memory_used_mb: number
  /** 内存总量（MB） */
  memory_total_mb: number
  /** 磁盘使用率 */
  disk_usage: number
  /** 请求延迟（ms） */
  request_latency_ms: number
  /** 每秒请求数 */
  requests_per_second: number
}

// ==================== 配置相关类型 ====================

/**
 * 系统配置项
 */
export interface ConfigItem {
  /** 配置键 */
  key: string
  /** 配置值 */
  value: unknown
  /** 配置类型 */
  type: 'string' | 'number' | 'boolean' | 'json'
  /** 配置描述 */
  description?: string
  /** 默认值 */
  default_value?: unknown
  /** 是否可编辑 */
  editable: boolean
  /** 配置分类 */
  category?: string
}

// ==================== UI 配置相关类型 ====================

/**
 * 应用配置
 */
export interface AppConfig {
  /** 应用名称 */
  name: string
  /** Logo 路径 */
  logo: string
  /** 版本号 */
  version: string
}

/**
 * 主题配置
 */
export interface ThemeConfig {
  /** 主题名称 */
  name: string
}

/**
 * 侧边栏菜单项配置
 */
export interface SidebarItemConfig {
  /** 显示名称 */
  name: string
  /** 图标名称 */
  icon: string
  /** 路由路径 */
  path: string
}

/**
 * QuickBar 子项配置
 */
export interface QuickChildItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示文本 */
  label: string
  /** 图标名称 */
  icon: string
}

/**
 * QuickBar 项配置
 */
export interface QuickItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示方式：visible-直接显示, dropdown-下拉菜单 */
  display: 'visible' | 'dropdown'
  /** 类型：action-动作, menu-菜单 */
  type: 'action' | 'menu'
  /** 图标名称 */
  icon: string
  /** 替代图标名称 */
  iconAlt?: string
  /** 显示文本 */
  label: string
  /** 子菜单项 */
  children?: QuickChildItemConfig[]
}

/**
 * DockBar 项配置
 */
export interface DockItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示文本 */
  label: string
  /** 图标名称 */
  icon: string
  /** 路由路径 */
  path: string
  /** 是否禁用 */
  disabled?: boolean
  /** 是否需要路由参数 */
  requiresParams?: boolean
  /** 参数名列表 */
  paramKeys?: string[]
  /** 缺少参数时的提示消息 */
  missingParamsMessage?: string
}

/**
 * DockBar 页面配置
 */
export interface DockPageConfig {
  /** 是否启用 */
  enabled: boolean
  /** 位置 */
  position: 'bottom' | 'top' | 'left' | 'right'
  /** 距离边缘偏移 */
  offset: number
  /** 菜单项列表 */
  items: DockItemConfig[]
}

/**
 * DockBar 配置
 */
export interface DockConfig {
  [pagePath: string]: DockPageConfig
}

/**
 * 分页结果
 */
export interface PaginationResult<T> {
  /** 数据列表 */
  items: T[]
  /** 总记录数 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页大小 */
  page_size: number
  /** 总页数 */
  total_pages: number
}

/**
 * 外观配置
 * 用户可自定义的应用外观设置
 */
export interface AppearanceConfig {
  /** 背景图片 URL */
  backgroundImage: string
  /** 背景图片不透明度 (0-1) */
  backgroundOpacity: number
  /** 强调色（hex 格式），null 表示使用主题默认色 */
  accentColor: string | null
}

/**
 * UI配置
 * 注意：应用配置(app)已从云端配置中移除，改为从 ui.ts 配置文件读取
 */
export interface UIConfig {
  /** 主题配置 */
  theme: ThemeConfig
  /** 侧边栏配置 */
  sidebar: { items: SidebarItemConfig[] }
  /** QuickBar 配置 */
  quickBar: QuickItemConfig[]
  /** DockBar 配置 */
  dock: DockConfig
  /** 外观配置 */
  appearance?: AppearanceConfig
}

/**
 * 保存UI配置参数
 * 注意：应用配置(app)已从云端配置中移除，改为从 ui.ts 配置文件读取
 */
export interface SaveUIConfigParams {
  theme?: ThemeConfig
  sidebar?: { items: SidebarItemConfig[] }
  quickBar?: QuickItemConfig[]
  dock?: DockConfig
}

/**
 * UI配置响应
 */
export interface UIConfigResponse {
  success: boolean
  data?: UIConfig
  error?: string
}

// ==================== WebSocket 类型 ====================

/**
 * WebSocket 消息类型
 */
export type WsMessageType =
  | 'Auth'
  | 'AuthResult'
  | 'Ping'
  | 'Pong'
  | 'Reconnect'
  | 'ReconnectResult'
  | 'Error'
  | 'JoinRoom'
  | 'LeaveRoom'
  | 'RoomJoined'
  | 'RoomLeft'
  | 'UserJoined'
  | 'UserLeft'
  | 'OnlineUsers'
  | 'ChatMessage'
  | 'NewMessage'
  | 'Typing'
  | 'StopTyping'
  // 通知系统消息类型
  | 'GetOfflineNotifications'
  | 'OfflineNotifications'
  | 'MarkNotificationRead'
  | 'MarkAllNotificationsRead'
  | 'NotificationReadConfirm'
  | 'PrivateMessage'
  | 'Mentioned'
  | 'RoomInvitation'
  | 'SystemNotification'
  | 'FileUploadComplete'
  | 'PendingAction'
  // 系统日志流消息类型
  | 'SubscribeLogs'
  | 'UnsubscribeLogs'
  | 'LogEntry'
  | 'LogSubscriptionConfirmed'

/**
 * WebSocket 消息基础接口
 */
export interface WsMessage<T = unknown> {
  /** 消息类型 */
  type: WsMessageType
  /** 消息载荷 */
  payload?: T
}

/**
 * WebSocket 认证消息
 */
export interface WsAuthMessage {
  /** 访问令牌 */
  token: string
}

/**
 * WebSocket 认证结果
 */
export interface WsAuthResult {
  /** 是否成功 */
  success: boolean
  /** 错误信息 */
  error?: string
  /** 用户ID */
  user_id?: string
}

/**
 * WebSocket 连接状态
 */
export type WsConnectionState = 'connecting' | 'connected' | 'authenticated' | 'disconnected' | 'reconnecting'

// ==================== 通知相关类型 ====================

/**
 * 通知数据库类型
 *
 * 与后端 HTTP API 返回的 notification_type 字段对应
 * 详见: docs/api/v1/http/notifications.md
 */
export type NotificationDbType =
  | 'mention'              // @提及通知
  | 'private_message'      // 私信通知
  | 'room_invitation'      // 房间邀请
  | 'system'               // 系统通知
  | 'file_upload'          // 文件上传完成
  | 'pending_action'       // 待办通知
  | 'config_reload_required' // 配置需要重新加载（保留用于兼容）
  | 'system_notification'  // 系统通知（旧类型，保留用于兼容）
  | 'file_upload_complete' // 文件上传完成（旧类型，保留用于兼容）
  | 'mentioned'            // @提及（旧类型，保留用于兼容）

/**
 * 通知项
 */
export interface NotificationItem {
  /** 通知ID */
  id: string
  /** 通知类型 */
  notification_type: NotificationDbType
  /** 标题 */
  title: string | null
  /** 内容 */
  content: string
  /** 附加数据 */
  data: Record<string, unknown> | null
  /** 是否已读 */
  is_read: boolean
  /** 读取时间 */
  read_at: string | null
  /** 创建时间 */
  created_at: string
}

/**
 * 获取离线通知参数
 */
export interface GetOfflineNotificationsParams {
  /** 上次获取的最后通知ID */
  last_notification_id?: string | null
  /** 获取数量限制 */
  limit?: number
}

/**
 * 离线通知响应
 */
export interface OfflineNotificationsPayload {
  /** 通知列表 */
  notifications: NotificationItem[]
  /** 是否还有更多 */
  has_more: boolean
}

/**
 * 标记通知已读参数
 */
export interface MarkNotificationReadParams {
  /** 通知ID */
  notification_id: string
}

/**
 * 通知已读确认
 */
export interface NotificationReadConfirmPayload {
  /** 通知ID */
  notification_id: string
}

// ==================== 通用响应类型 ====================

/**
 * API 通用响应
 */
export interface ApiResponse<T> {
  /** 是否成功 */
  success: boolean
  /** 响应数据 */
  data?: T
  /** 错误码 */
  code?: string
  /** 错误信息 */
  error?: string
  /** 错误详情 */
  message?: string
}

/**
 * 分页参数
 */
export interface PaginationParams {
  /** 页码 */
  page?: number
  /** 每页数量 */
  page_size?: number
  /** 限制数量 */
  limit?: number
  /** 偏移量 */
  offset?: number
}

/**
 * 分页响应
 */
export interface PaginatedResponse<T> {
  /** 数据列表 */
  items: T[]
  /** 总数量 */
  total: number
  /** 页码 */
  page: number
  /** 每页数量 */
  page_size: number
  /** 总页数 */
  total_pages: number
}

/**
 * 主题类型（用于UI配置）
 */
export type ThemeType = 'light' | 'dark' | 'system'

// ==================== 系统日志流类型 ====================

/**
 * 日志级别
 */
export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'all'

/**
 * 日志模块
 */
export type LogModule = 'websocket' | 'room' | 'message' | 'performance' | 'all'

/**
 * 系统日志条目
 */
export interface LogEntry {
  /** 日志级别 */
  level: LogLevel
  /** 日志模块/目标 */
  target: string
  /** 日志消息内容 */
  message: string
  /** 时间戳 */
  timestamp: string
  /** 结构化日志字段 */
  fields?: Record<string, unknown>
}

/**
 * 订阅日志参数
 */
export interface SubscribeLogsParams {
  /** 日志级别过滤 */
  level?: LogLevel
  /** 模块过滤 */
  module?: LogModule
}

/**
 * 日志订阅确认
 */
export interface LogSubscriptionConfirmedPayload {
  /** 是否成功 */
  success: boolean
  /** 确认消息 */
  message: string
}
