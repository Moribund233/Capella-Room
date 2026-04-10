/**
 * API 模块统一导出
 */

// HTTP 客户端（纯请求封装，不处理业务逻辑）
export { apiClient } from './client'

// Token 管理（存储、获取、清除）
export {
  getAccessToken,
  getRefreshToken,
  setTokens,
  clearTokens,
  setUser,
  getStoredUser,
  clearUser,
  isAuthenticated,
} from './token'

// 认证相关 API（登录、注册、登出等业务逻辑）
export {
  login,
  register,
  logout,
  refreshToken,
  getCurrentUser,
} from './auth'

// WebSocket 客户端
export { wsClient, WebSocketClient } from './websocket'

// 房间管理 API
export {
  getRooms,
  getRoomDetail,
  createRoom,
  updateRoom,
  deleteRoom,
  joinRoom,
  leaveRoom,
  getMyRooms,
  type RoomListResponse,
  type CreateRoomRequest,
  type UpdateRoomRequest,
  type RoomMember,
  type RoomDetailResponse,
} from './room'

// 从 types/api 重新导出 Room 类型
export type { Room } from '@/types/api'
