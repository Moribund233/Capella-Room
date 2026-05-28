import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type {
  UserSettings,
  PartialUserSettings,
  LoginDevice,
  LoginHistory,
  RoomUserSettings,
} from '@/types/settings'
import {
  toBackendSettings,
  fromBackendSettings,
  type BackendUserSettings,
} from '@/utils/settingsTransform'

/**
 * 用户设置 API
 * 提供用户个性化设置的 CRUD 操作
 * 自动处理 camelCase <-> snake_case 字段转换
 */
export const settingsApi = {
  /**
   * 获取当前用户的完整设置
   * @returns 用户设置
   */
  async getSettings(): Promise<ApiResponse<UserSettings>> {
    // 注意：httpClient 的响应拦截器已经提取了 response.data
    const response = (await httpClient.get<unknown>('/users/me/settings')) as unknown as ApiResponse<BackendUserSettings>
    // 转换后端字段为前端字段
    if (response.data) {
      return {
        success: response.success,
        data: fromBackendSettings(response.data) as UserSettings,
        code: response.code,
        message: response.message,
      }
    }
    // 如果 response.data 不存在，说明后端返回了错误
    return {
      success: response.success ?? false,
      data: undefined as unknown as UserSettings,
      code: response.code,
      error: response.error || response.message || '获取设置失败',
      message: response.message,
    }
  },

  /**
   * 更新用户设置（支持部分更新）
   * @param settings 部分设置对象
   * @returns 更新后的设置
   */
  async updateSettings(settings: PartialUserSettings): Promise<ApiResponse<UserSettings>> {
    // 转换前端字段为后端字段
    const backendSettings = toBackendSettings(settings)
    console.log('[API] Sending settings update:', backendSettings)
    // 注意：httpClient 的响应拦截器已经提取了 response.data
    // 所以这里的 response 实际上是 ApiResponse<BackendUserSettings>
    const response = (await httpClient.patch<unknown>(
      '/users/me/settings',
      backendSettings
    )) as unknown as ApiResponse<BackendUserSettings>
    console.log('[API] Received response:', response)
    // 转换后端响应为前端字段
    if (response.data) {
      return {
        success: response.success,
        data: fromBackendSettings(response.data) as UserSettings,
        code: response.code,
        message: response.message,
      }
    }
    // 如果 response.data 不存在，说明后端返回了错误
    return {
      success: response.success ?? false,
      data: undefined as unknown as UserSettings,
      code: response.code,
      error: response.error || response.message || '保存设置失败',
      message: response.message,
    }
  },

  /**
   * 重置用户设置为默认值
   * @returns 重置后的默认设置
   */
  async resetSettings(): Promise<ApiResponse<UserSettings>> {
    // 注意：httpClient 的响应拦截器已经提取了 response.data
    const response = (await httpClient.delete<unknown>('/users/me/settings')) as unknown as ApiResponse<BackendUserSettings>
    // 转换后端字段为前端字段
    if (response.data) {
      return {
        success: response.success,
        data: fromBackendSettings(response.data) as UserSettings,
        code: response.code,
        message: response.message,
      }
    }
    // 如果 response.data 不存在，说明后端返回了错误
    return {
      success: response.success ?? false,
      data: undefined as unknown as UserSettings,
      code: response.code,
      error: response.error || response.message || '重置设置失败',
      message: response.message,
    }
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
    return httpClient.post(`/users/me/devices/${deviceId}/logout`)
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
    return httpClient.post('/users/me/devices/logout-all-others')
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
    return httpClient.get('/users/me/room-settings')
  },

  /**
   * 获取指定房间的个性化设置
   * @param roomId 房间ID
   * @returns 房间设置
   */
  getRoomSetting(roomId: string): Promise<ApiResponse<RoomUserSettings>> {
    return httpClient.get(`/users/me/room-settings/${roomId}`)
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
    return httpClient.patch(`/users/me/room-settings/${roomId}`, settings)
  },

  /**
   * 重置指定房间的个性化设置
   * @param roomId 房间ID
   * @returns 重置后的设置
   */
  resetRoomSetting(roomId: string): Promise<ApiResponse<RoomUserSettings>> {
    return httpClient.delete(`/users/me/room-settings/${roomId}`)
  },
}
