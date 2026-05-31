import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type {
  UserSettings,
  PartialUserSettings,
  LoginDevice,
  LoginHistory,
  RoomUserSettings,
} from '@/types/settings'

/**
 * 用户设置 API
 * 提供用户个性化设置的 CRUD 操作
 */
export const settingsApi = {
  /**
   * 获取当前用户的完整设置
   * @returns 用户设置
   */
  getSettings(): Promise<ApiResponse<UserSettings>> {
    return httpClient.get('/users/me/settings')
  },

  /**
   * 更新用户设置（支持部分更新）
   * @param settings 部分设置对象
   * @returns 更新后的设置
   */
  updateSettings(settings: PartialUserSettings): Promise<ApiResponse<UserSettings>> {
    return httpClient.patch('/users/me/settings', settings)
  },

  /**
   * 重置用户设置为默认值
   * @returns 重置后的默认设置
   */
  resetSettings(): Promise<ApiResponse<UserSettings>> {
    return httpClient.delete('/users/me/settings')
  },
}

/**
 * 账号安全 API
 * 提供登录设备管理和登录历史查询
 */
export const securityApi = {
  /**
   * 获取当前登录设备列表
   * @returns 设备列表
   */
  getLoginDevices(): Promise<ApiResponse<{ devices: LoginDevice[] }>> {
    return httpClient.get('/users/me/devices')
  },

  /**
   * 登出指定设备
   * @param deviceId 设备ID
   * @returns 操作结果
   */
  logoutDevice(deviceId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/users/me/devices/${deviceId}`)
  },

  /**
   * 禁用指定设备
   * @param deviceId 设备ID
   * @returns 操作结果
   */
  blockDevice(deviceId: string): Promise<ApiResponse<void>> {
    return httpClient.post(`/users/me/devices/${deviceId}/block`)
  },

  /**
   * 启用被禁用的设备
   * @param deviceId 设备ID
   * @returns 操作结果
   */
  unblockDevice(deviceId: string): Promise<ApiResponse<void>> {
    return httpClient.post(`/users/me/devices/${deviceId}/unblock`)
  },

  /**
   * 登出所有其他设备
   * @returns 操作结果
   */
  logoutAllOtherDevices(): Promise<ApiResponse<void>> {
    return httpClient.delete('/users/me/devices')
  },

  /**
   * 获取登录历史记录
   * @param params 查询参数
   * @returns 登录历史列表
   */
  getLoginHistory(params?: { limit?: number; offset?: number }): Promise<ApiResponse<{
    history: LoginHistory[]
    total: number
    hasMore: boolean
  }>> {
    return httpClient.get('/users/me/login-history', { params })
  },
}

/**
 * 房间级用户设置 API
 * 提供针对特定房间的个性化设置
 */
export const roomSettingsApi = {
  /**
   * 获取所有房间的个性化设置
   * @returns 房间设置列表
   */
  getRoomSettings(): Promise<ApiResponse<{ settings: RoomUserSettings[] }>> {
    return httpClient.get('/users/me/rooms/settings')
  },

  /**
   * 获取指定房间的个性化设置
   * @param roomId 房间ID
   * @returns 房间设置
   */
  getRoomSetting(roomId: string): Promise<ApiResponse<RoomUserSettings>> {
    return httpClient.get(`/users/me/rooms/${roomId}/settings`)
  },

  /**
   * 更新指定房间的个性化设置
   * @param roomId 房间ID
   * @param settings 部分房间设置
   * @returns 更新后的设置
   */
  updateRoomSetting(
    roomId: string,
    settings: Partial<Omit<RoomUserSettings, 'roomId' | 'roomName'>>
  ): Promise<ApiResponse<RoomUserSettings>> {
    return httpClient.patch(`/users/me/rooms/${roomId}/settings`, settings)
  },

  /**
   * 重置指定房间的个性化设置
   * @param roomId 房间ID
   * @returns 重置后的设置
   */
  resetRoomSetting(roomId: string): Promise<ApiResponse<RoomUserSettings>> {
    return httpClient.delete(`/users/me/rooms/${roomId}/settings`)
  },
}
