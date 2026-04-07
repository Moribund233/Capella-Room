/**
 * 认证相关 API
 */
import { post, type ApiResponse } from './client'

/** 用户角色 */
export type UserRole = 'user' | 'admin' | 'super_admin'

/** 用户状态 */
export interface User {
  id: string
  username: string
  email: string
  avatar_url: string | null
  status: 'online' | 'offline' | 'away'
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

/**
 * 用户登录
 * @param credentials 登录凭证
 * @returns 登录响应，包含 Token 和用户信息
 */
export function login(credentials: LoginRequest): Promise<ApiResponse<LoginResponse>> {
  return post<LoginResponse>('/auth/login', credentials, { skipAuth: true })
}

/**
 * 刷新访问令牌
 * @param refreshToken 刷新令牌
 * @returns 新的 Token 信息
 */
export function refreshToken(refreshToken: string): Promise<ApiResponse<RefreshTokenResponse>> {
  return post<RefreshTokenResponse>('/auth/refresh', { refresh_token: refreshToken }, { skipAuth: true })
}

/**
 * 用户登出
 * 使当前访问令牌失效
 */
export function logout(): Promise<ApiResponse<void>> {
  return post<void>('/auth/logout', {})
}

/**
 * 检查用户是否为管理员
 * @param role 用户角色
 * @returns 是否为管理员
 */
export function isAdmin(role: UserRole): boolean {
  return role === 'admin' || role === 'super_admin'
}

/**
 * 检查用户是否为超级管理员
 * @param role 用户角色
 * @returns 是否为超级管理员
 */
export function isSuperAdmin(role: UserRole): boolean {
  return role === 'super_admin'
}
