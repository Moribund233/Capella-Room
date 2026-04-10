/**
 * API 类型定义
 */

// 通用响应格式
export interface ApiResponse<T> {
  code: number
  message: string
  data: T
}

// 用户信息
export interface User {
  id: string
  username: string
  email: string
  role: 'user' | 'admin' | 'super_admin'
  status: 'active' | 'inactive'
  created_at: string
  last_login?: string
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
  owner_id: string
}

// 消息信息
export interface Message {
  id: string
  content: string
  sender: string
  sender_id: string
  room_id: string
  created_at: string
  type: 'text' | 'image' | 'file'
}

// WebSocket 消息
export interface WebSocketMessage {
  type: string
  [key: string]: any
}
