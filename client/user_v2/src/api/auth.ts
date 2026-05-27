import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { LoginCredentials, RegisterData, User } from '@/types/user'

interface LoginResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  token_type: string
  user: User
}

interface RefreshTokenResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  token_type: string
}

export const authApi = {
  /**
   * 用户登录
   */
  login(credentials: LoginCredentials): Promise<ApiResponse<LoginResponse>> {
    return httpClient.post('/auth/login', credentials)
  },

  /**
   * 用户注册
   */
  register(data: RegisterData): Promise<ApiResponse<User>> {
    return httpClient.post('/auth/register', data)
  },

  /**
   * 刷新 Token
   */
  refresh(refreshToken: string): Promise<ApiResponse<RefreshTokenResponse>> {
    return httpClient.post('/auth/refresh', { refresh_token: refreshToken })
  },

  /**
   * 获取当前用户信息
   */
  getMe(): Promise<ApiResponse<User>> {
    return httpClient.get('/users/me')
  },

  /**
   * 登出
   */
  logout(): Promise<ApiResponse<void>> {
    return httpClient.post('/users/logout')
  },
}
