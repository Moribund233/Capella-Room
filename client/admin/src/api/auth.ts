import { http } from './request'
import type { ApiResponse, LoginParams, LoginResult, RegisterParams, UserInfo } from '@/types'

/**
 * 认证相关接口
 */
export const authApi = {
  /**
   * 用户登录
   * @param params 登录参数（邮箱和密码）
   * @returns 登录结果，包含访问令牌、刷新令牌和用户信息
   */
  login(params: LoginParams): Promise<ApiResponse<LoginResult>> {
    return http.post<LoginResult>('/auth/login', params, { requireAuth: false })
  },

  /**
   * 用户注册
   * @param params 注册参数（用户名、邮箱和密码）
   * @returns 注册结果，包含新创建的用户信息
   */
  register(params: RegisterParams): Promise<ApiResponse<UserInfo>> {
    return http.post<UserInfo>('/auth/register', params, { requireAuth: false })
  },

  /**
   * 刷新访问令牌
   * @param refreshToken 刷新令牌
   * @returns 新的令牌对
   */
  refreshToken(refreshToken: string): Promise<ApiResponse<LoginResult>> {
    return http.post<LoginResult>('/auth/refresh', { refresh_token: refreshToken }, { requireAuth: false })
  },

  /**
   * 获取当前登录用户信息
   * @returns 当前用户信息
   */
  getCurrentUser(): Promise<ApiResponse<UserInfo>> {
    return http.get<UserInfo>('/users/me')
  },
}

/**
 * 检查用户是否为管理员（admin 或 super_admin）
 * @param role 用户角色
 * @returns 是否为管理员
 */
export function isAdmin(role: string | undefined): boolean {
  return role === 'admin' || role === 'super_admin'
}

/**
 * 检查用户是否为超级管理员
 * @param role 用户角色
 * @returns 是否为超级管理员
 */
export function isSuperAdmin(role: string | undefined): boolean {
  return role === 'super_admin'
}
