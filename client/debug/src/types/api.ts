/**
 * API 类型定义
 */

// 通用响应格式
// 后端返回格式: { success: bool, data?: T, message?: string }
export interface ApiResponse<T> {
  success: boolean
  data: T
  message?: string
}

// 用户信息（完整）
export interface User {
  id: string
  username: string
  email: string
  role: 'user' | 'admin' | 'super_admin'
  status: 'active' | 'inactive'
  created_at: string
  last_login?: string
}

// 用户信息（简化版，用于嵌套）
export interface UserInfo {
  id: string
  username: string
  avatar_url?: string
}

// 发送者信息
export interface SenderInfo {
  id: string
  username: string
  avatar_url?: string
}

// 登录请求
export interface LoginRequest {
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

// 注册请求
export interface RegisterRequest {
  username: string
  email: string
  password: string
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
  owner: UserInfo  // 修改：从 owner_id 改为 owner
}

// 消息信息
export interface Message {
  id: string
  content: string
  sender: SenderInfo  // 修改：从 string 改为 SenderInfo 对象
  room_id: string
  created_at: string
  type: 'text' | 'image' | 'file'
  reply_to?: string  // 回复的消息ID
}

// 文件资源信息
export interface FileResource {
  id: string
  original_name: string
  file_url: string
  file_size: number
  mime_type: string
  category: 'image' | 'document' | 'video' | 'audio' | 'other'
  usage_type: 'avatar' | 'message' | 'room_cover' | 'general'
  uploader?: UserInfo  // 新增：上传者信息
  created_at: string
}

// 审计告警信息
export interface AuditAlert {
  id: string
  rule_id?: string
  alert_type: string
  severity: 'info' | 'warning' | 'error' | 'critical'
  title: string
  description: string
  affected_user?: UserInfo  // 新增：受影响用户信息
  acknowledged_by?: UserInfo  // 新增：确认者信息
  resolved_by?: UserInfo  // 新增：解决者信息
  status: 'new' | 'acknowledged' | 'resolved' | 'ignored'
  created_at: string
  updated_at: string
}

// WebSocket 消息
export interface WebSocketMessage {
  type: string
  [key: string]: any
}
