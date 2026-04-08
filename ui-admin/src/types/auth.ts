/**
 * 认证模块类型定义
 */

/** 用户角色 */
export type UserRole = 'user' | 'admin' | 'super_admin'

/** 用户状态 */
export type UserStatus = 'online' | 'offline' | 'away'

/** 用户 */
export interface User {
  id: string
  username: string
  email: string
  avatar_url: string | null
  status: UserStatus
  role: UserRole
  created_at: string
}

/** 登录请求参数 */
export interface LoginRequest {
  email: string
  password: string
}

/** 登录响应数据 */
export interface LoginResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  token_type: string
  user: User
}

/** 刷新 Token 请求 */
export interface RefreshTokenRequest {
  refresh_token: string
}

/** 刷新 Token 响应 */
export interface RefreshTokenResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  token_type: string
}
