import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { DirectRoom, CreateDirectRoomData } from '@/types/room'

/**
 * 私聊房间 API
 */
export const directRoomApi = {
  /**
   * 创建或获取私聊房间
   * @param targetUserId - 对方用户ID
   * @returns 私聊房间信息
   */
  createDirectRoom(targetUserId: string): Promise<ApiResponse<DirectRoom>> {
    const data: CreateDirectRoomData = { target_user_id: targetUserId }
    return httpClient.post('/rooms/direct', data)
  },

  /**
   * 获取私聊房间列表
   * @returns 私聊房间列表
   */
  getDirectRooms(): Promise<ApiResponse<DirectRoom[]>> {
    return httpClient.get('/rooms/direct/list')
  },
}
