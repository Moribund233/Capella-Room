import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { User } from '@/types/user'
import type { Room } from '@/types/room'

export interface UserStats {
  joined_rooms: number
  total_messages: number
  online_hours: number
}

export interface UpdateProfileData {
  username?: string
  avatar_url?: string | null
}

export interface ChangePasswordData {
  old_password: string
  new_password: string
}

/**
 * 用户相关 API
 */
export const userApi = {
  /**
   * 获取当前用户信息
   */
  getMe(): Promise<ApiResponse<User>> {
    return httpClient.get('/users/me')
  },

  /**
   * 更新当前用户信息
   * @param data 更新数据
   */
  updateProfile(data: UpdateProfileData): Promise<ApiResponse<User>> {
    return httpClient.put('/users/me', data)
  },

  /**
   * 修改密码
   * @param data 密码数据
   */
  changePassword(data: { oldPassword: string; newPassword: string }): Promise<ApiResponse<string>> {
    return httpClient.put('/users/me/password', {
      old_password: data.oldPassword,
      new_password: data.newPassword
    })
  },

  /**
   * 获取用户统计信息
   */
  getStats(): Promise<ApiResponse<UserStats>> {
    return httpClient.get('/users/me/stats')
  },

  /**
   * 获取我的聊天室列表
   */
  getMyRooms(): Promise<ApiResponse<{ rooms: Room[]; total: number }>> {
    return httpClient.get('/users/me/rooms')
  },

  /**
   * 获取用户列表
   * @param params 查询参数
   */
  getUsers(params?: { limit?: number; offset?: number }): Promise<ApiResponse<{ users: User[]; total: number }>> {
    return httpClient.get('/users', { params })
  },

  /**
   * 获取指定用户信息
   * @param userId 用户ID
   */
  getUser(userId: string): Promise<ApiResponse<User>> {
    return httpClient.get(`/users/${userId}`)
  },

  /**
   * 删除当前账号（自服务软删除）
   */
  deleteAccount(): Promise<ApiResponse<void>> {
    return httpClient.delete('/users/me')
  }
}
