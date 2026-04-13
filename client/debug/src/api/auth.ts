/**
 * 认证相关 API
 * 负责处理登录、注册、登出等认证业务逻辑，包括 token 管理
 */

import { apiClient } from './client'
import {
  setTokens,
  clearTokens,
  setUser,
  getStoredUser,
  isAuthenticated,
  getRefreshToken,
} from './token'
import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
  RefreshTokenResponse,
  User,
} from '@/types/api'

// API 基础配置（用于刷新 token 的原始请求）
const API_BASE_URL = import.meta.env.VITE_BACKEND_URL || 'http://localhost:8080'

/**
 * 用户登录
 * @param credentials 登录凭证
 * @returns 登录响应
 */
export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  const response = await apiClient.post<LoginResponse>('/api/v1/auth/login', credentials)

  // 保存 token 和用户信息
  if (response.data.access_token && response.data.refresh_token) {
    setTokens(response.data.access_token, response.data.refresh_token)
    setUser(response.data.user)
  }

  return response.data
}

/**
 * 用户注册
 * @param data 注册信息
 * @returns 注册响应
 */
export async function register(data: RegisterRequest): Promise<RegisterResponse> {
  const response = await apiClient.post<RegisterResponse>('/api/v1/auth/register', data)

  // 保存 token 和用户信息
  if (response.data.access_token && response.data.refresh_token) {
    setTokens(response.data.access_token, response.data.refresh_token)
    setUser(response.data.user)
  }

  return response.data
}

/**
 * 刷新 Token
 * @returns 是否刷新成功
 */
export async function refreshToken(): Promise<boolean> {
  const refreshToken = getRefreshToken()
  if (!refreshToken) {
    return false
  }

  try {
    const response = await fetch(`${API_BASE_URL}/api/v1/auth/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
    })

    if (response.ok) {
      const data: { data: RefreshTokenResponse } = await response.json()
      setTokens(data.data.access_token, data.data.refresh_token)
      return true
    }
  } catch {
    // 刷新失败
  }

  return false
}

/**
 * 用户登出
 */
export async function logout(): Promise<void> {
  try {
    await apiClient.post('/api/v1/users/logout')
  } finally {
    clearTokens()
  }
}

/**
 * 获取当前用户信息
 * @returns 用户信息
 */
export async function getCurrentUser(): Promise<User> {
  const response = await apiClient.get<User>('/api/v1/users/me')

  // 更新本地存储的用户信息
  setUser(response.data)

  return response.data
}

// 重新导出 token 模块的函数，方便使用
export { getStoredUser, isAuthenticated }
