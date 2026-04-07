/**
 * API 模块入口
 * 统一导出所有 API 相关功能
 */

// Client
export {
  request,
  get,
  post,
  put,
  del,
  ApiError,
  type ApiResponse,
} from './client'

// Auth
export {
  login,
  logout,
  refreshToken,
  isAdmin,
  isSuperAdmin,
  type User,
  type UserRole,
  type LoginRequest,
  type LoginResponse,
  type RefreshTokenRequest,
  type RefreshTokenResponse,
} from './auth'
