/**
 * API 类型定义
 * 用户客户端 API 类型
 */

// 通用响应格式
export interface ApiResponse<T> {
  success: boolean
  data: T
  message?: string
}

// 分页响应
export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  per_page: number
}

// 用户在线状态
export type UserOnlineStatus = 'online' | 'offline' | 'away' | 'disabled'

// 用户角色
export type UserRole = 'user' | 'admin' | 'super_admin'

// 用户信息
export interface User {
  id: string
  username: string
  email: string
  role: UserRole
  status: UserOnlineStatus
  created_at: string
  updated_at: string
  avatar_url?: string
}

// 用户信息（简化版）
export interface UserInfo {
  id: string
  username: string
  avatar_url?: string
}

// 登录请求
export interface LoginRequest {
  email: string
  password: string
}

// 注册请求
export interface RegisterRequest {
  username: string
  email: string
  password: string
}

// 登录响应
export interface LoginResponse {
  user: User
  access_token: string
  refresh_token: string
  expires_in: number
}

// 注册响应
export interface RegisterResponse {
  user: User
  access_token: string
  refresh_token: string
  expires_in: number
}

// Token 刷新响应
export interface RefreshTokenResponse {
  access_token: string
  refresh_token: string
  expires_in: number
}

// 房间信息
export interface Room {
  id: string
  name: string
  description?: string
  is_private: boolean
  max_members: number
  member_count: number
  created_at: string
  updated_at: string
  owner: UserInfo
}

// 消息信息
export interface Message {
  id: string
  content: string
  sender: UserInfo
  room_id: string
  created_at: string
  type: 'text' | 'image' | 'file'
  reply_to?: string
}

// 文件资源信息
export interface FileResource {
  id: string
  filename: string
  url: string
  size: number
  mime_type: string
  created_at: string
}

// 房间列表响应
export interface RoomListResponse {
  items: Room[]
  total: number
  page: number
  per_page: number
}

// 创建房间请求
export interface CreateRoomRequest {
  name: string
  description?: string
  is_private?: boolean
  max_members?: number
}

// 更新房间请求
export interface UpdateRoomRequest {
  name?: string
  description?: string
  is_private?: boolean
  max_members?: number
}

// 分页请求参数
export interface PaginationParams {
  page?: number
  per_page?: number
  page_size?: number
}

// 系统状态响应
export interface SystemStatus {
  status: 'healthy' | 'degraded' | 'unhealthy'
  version: string
  timestamp: string
  uptime: number
}

// 系统统计信息
export interface SystemStats {
  total_users: number
  total_rooms: number
  total_messages: number
  online_users: number
}

// 客户端配置
export interface ClientConfig {
  websocket: {
    heartbeat_interval_secs: number
    heartbeat_timeout_secs: number
    auth_timeout_secs: number
    max_reconnect_attempts?: number
    reconnect_interval_ms?: number
    heartbeat_interval_ms?: number
  }
  reconnect: {
    base_delay_ms: number
    max_delay_ms: number
    max_attempts: number
    multiplier: number
  }
  upload: {
    max_file_size: number
    max_file_size_human: string
  }
  system: {
    maintenance_mode: boolean
    maintenance_message?: string
  }
}
