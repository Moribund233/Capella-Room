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
  /** 过期时间（秒） */
  expires_in: number
  /** 令牌类型 */
  token_type: string
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

// ==================== API 相关类型 ====================

/**
 * API响应类型
 */
export interface ApiResponse<T = unknown> {
  /** 是否成功 */
  success: boolean
  /** 状态码 */
  code?: string | number
  /** 错误类型 */
  error?: string
  /** 响应消息 */
  message?: string
  /** 响应数据 */
  data?: T
}

/**
 * 分页参数类型
 */
export interface PaginationParams {
  /** 当前页码 */
  page?: number
  /** 每页条数 */
  page_size?: number
  /** 搜索关键词 */
  search?: string
}

/**
 * 分页结果类型
 */
export interface PaginationResult<T> {
  /** 数据列表 */
  list: T[]
  /** 总条数 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页条数 */
  page_size: number
}

// ==================== UI 配置类型 ====================

/**
 * 侧边栏菜单项配置
 */
export interface SidebarItemConfig {
  /** 菜单名称 */
  name: string
  /** 图标名称（Lucide图标） */
  icon: string
  /** 路由路径 */
  path: string
}

/**
 * Dock 菜单项配置
 */
export interface DockItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 图标名称 */
  icon: string
  /** 路由路径 */
  path: string
  /** 是否禁用 */
  disabled?: boolean
}

/**
 * 页面 Dock 配置
 */
export interface DockPageConfig {
  /** 是否启用 */
  enabled: boolean
  /** 位置 */
  position: 'bottom' | 'top' | 'left' | 'right'
  /** 偏移量 */
  offset: number
  /** 菜单项 */
  items: DockItemConfig[]
}

/**
 * Dock 配置
 */
export type DockConfig = Record<string, DockPageConfig>

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
  name: Theme
}

/**
 * QuickBar 子项配置
 */
export interface QuickChildItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
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
  /** 显示方式 */
  display: 'visible' | 'dropdown'
  /** 类型 */
  type: 'action' | 'menu'
  /** 图标名称 */
  icon: string
  /** 替代图标（状态切换时使用） */
  iconAlt?: string
  /** 提示文本 */
  label: string
  /** 子菜单 */
  children?: QuickChildItemConfig[]
}

/**
 * 主题类型
 */
export type ThemeType = 'light' | 'dark'

/**
 * UI 配置
 */
export interface UIConfig {
  /** 应用配置 */
  app: AppConfig
  /** 侧边栏配置 */
  sidebar: {
    items: SidebarItemConfig[]
  }
  /** 主题配置 */
  theme: ThemeConfig
  /** Dock 配置 */
  dock: DockConfig
  /** QuickBar 配置 */
  quickBar?: QuickItemConfig[]
}

/**
 * UI 配置 API 响应数据
 */
export interface UIConfigResponse {
  /** 应用配置 */
  app?: Partial<AppConfig>
  /** 主题配置 */
  theme?: Partial<ThemeConfig>
  /** 侧边栏配置 */
  sidebar?: {
    items?: SidebarItemConfig[]
  }
  /** QuickBar 配置 */
  quickBar?: QuickItemConfig[]
  /** Dock 配置 */
  dock?: DockConfig
}

/**
 * 保存 UI 配置请求参数
 */
export interface SaveUIConfigParams {
  /** 应用配置 */
  app?: Partial<AppConfig>
  /** 主题配置 */
  theme?: Partial<ThemeConfig>
  /** 侧边栏配置 */
  sidebar?: {
    items?: SidebarItemConfig[]
  }
  /** QuickBar 配置 */
  quickBar?: QuickItemConfig[]
  /** Dock 配置 */
  dock?: DockConfig
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
