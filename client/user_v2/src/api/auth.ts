import axios from 'axios'
import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type {
  LoginCredentials, RegisterData, User, AuthData,
} from '@/types/user'

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

const v2Base = import.meta.env.VITE_API_BASE_URL.replace('/v1', '/v2')

const v2Client = axios.create({
  baseURL: v2Base,
  timeout: 10000,
  headers: { 'Content-Type': 'application/json' },
})

v2Client.interceptors.response.use(
  (response) => response.data,
  (error) => Promise.reject(error),
)

export const authApi = {
  /** v1: 密码登录 */
  login(credentials: LoginCredentials): Promise<ApiResponse<LoginResponse>> {
    return httpClient.post('/auth/login', credentials)
  },

  /** v2: 发送登录验证码 */
  loginSendCode(email: string): Promise<ApiResponse<unknown>> {
    return v2Client.post('/auth/login/send-code', { email })
  },

  /** v2: 验证码登录 */
  loginWithCode(email: string, code: string): Promise<ApiResponse<AuthData>> {
    return v2Client.post('/auth/login', { email, code })
  },

  /** v2: 发送注册验证码 */
  registerSendCode(email: string): Promise<ApiResponse<unknown>> {
    return v2Client.post('/auth/register/send-code', { email })
  },

  /** v2: 注册 */
  register(data: RegisterData): Promise<ApiResponse<AuthData>> {
    return v2Client.post('/auth/register', data)
  },

  /** v2: 发送重置密码验证码 */
  resetPasswordSendCode(email: string): Promise<ApiResponse<unknown>> {
    return v2Client.post('/auth/reset-password/send-code', { email })
  },

  /** v2: 重置密码 */
  resetPassword(email: string, code: string, password: string): Promise<ApiResponse<unknown>> {
    return v2Client.post('/auth/reset-password', { email, code, password })
  },

  refresh(refreshToken: string): Promise<ApiResponse<RefreshTokenResponse>> {
    return httpClient.post('/auth/refresh', { refresh_token: refreshToken })
  },

  getMe(): Promise<ApiResponse<User>> {
    return httpClient.get('/users/me')
  },

  logout(): Promise<ApiResponse<void>> {
    return httpClient.post('/users/logout')
  },
}
