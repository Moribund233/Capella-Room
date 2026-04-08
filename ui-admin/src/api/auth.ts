/**
 * 认证相关 API
 */
import { post, type ApiResponse } from './client'
import type {
  UserRole,
  User,
  LoginRequest,
  LoginResponse,
  RefreshTokenRequest,
  RefreshTokenResponse,
} from '@/types'

export type { UserRole, User, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse }

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
