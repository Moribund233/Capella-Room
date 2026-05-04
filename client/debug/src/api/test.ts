/**
 * 测试专用 API
 * 用于多用户测试、批量操作等测试场景
 * 注意：这些 API 不保存 token 到 localStorage，避免干扰主应用
 */

import { API_BASE_URL } from './client'
import type { LoginResponse, RegisterRequest, User } from '@/types/api'

/**
 * 测试用户登录（不保存 token）
 * @param username 用户名
 * @param password 密码
 * @returns 登录响应，包含 token 和用户信息
 */
export async function testUserLogin(
  username: string,
  password: string
): Promise<LoginResponse | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    })

    if (!response.ok) {
      return null
    }

    const result = await response.json()
    return result.data as LoginResponse
  } catch {
    return null
  }
}

/**
 * 测试用户注册（不保存 token）
 * @param data 注册信息
 * @returns 注册响应，包含 token 和用户信息
 */
export async function testUserRegister(
  data: RegisterRequest
): Promise<LoginResponse | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })

    if (!response.ok) {
      return null
    }

    const result = await response.json()
    return result.data as LoginResponse
  } catch {
    return null
  }
}

/**
 * 刷新 Token（用于测试用户）
 * @param refreshToken 刷新令牌
 * @returns 新的 token 信息
 */
export async function testRefreshToken(
  refreshToken: string
): Promise<{ access_token: string; refresh_token: string } | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
    })

    if (!response.ok) {
      return null
    }

    const result = await response.json()
    return result.data as { access_token: string; refresh_token: string }
  } catch {
    return null
  }
}

/**
 * 获取当前用户信息（使用指定 token）
 * @param accessToken 访问令牌
 * @returns 用户信息
 */
export async function testGetCurrentUser(accessToken: string): Promise<User | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/users/me`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      return null
    }

    const result = await response.json()
    return result.data as User
  } catch {
    return null
  }
}

/**
 * 用户登出（使用指定 token）
 * @param accessToken 访问令牌
 */
export async function testLogout(accessToken: string): Promise<boolean> {
  try {
    const response = await fetch(`${API_BASE_URL}/users/logout`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    })

    return response.ok
  } catch {
    return false
  }
}

/**
 * 生成随机测试用户信息
 * @param index 用户索引，用于生成唯一用户名
 * @returns 注册请求数据
 */
export function generateTestUser(index: number): RegisterRequest {
  const timestamp = Date.now().toString(36)
  const random = Math.random().toString(36).substring(2, 6)
  const uniqueId = `${timestamp}_${index}_${random}`

  return {
    username: `test_${uniqueId}`,
    email: `test_${uniqueId}@example.com`,
    password: 'Test123456!',
  }
}

/**
 * 解析 JWT Token 获取过期时间
 * @param token JWT Token
 * @returns 过期时间戳（毫秒），解析失败返回 null
 */
export function parseTokenExpiry(token: string): number | null {
  try {
    const parts = token.split('.')
    if (parts.length < 2) return null
    const base64Url = parts[1]!
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join('')
    )
    const payload = JSON.parse(jsonPayload)
    return payload.exp ? payload.exp * 1000 : null
  } catch {
    return null
  }
}

/**
 * 检查 token 是否即将过期
 * @param token JWT Token
 * @param thresholdMinutes 提前多少分钟认为即将过期，默认 5 分钟
 * @returns 是否即将过期
 */
export function isTokenExpiringSoon(token: string, thresholdMinutes: number = 5): boolean {
  const expiry = parseTokenExpiry(token)
  if (!expiry) return true

  const thresholdMs = thresholdMinutes * 60 * 1000
  return Date.now() + thresholdMs >= expiry
}
