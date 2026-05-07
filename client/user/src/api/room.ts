import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { Room, RoomMember, CreateRoomData, UpdateRoomData, ListRoomsParams, DirectRoom, CreateDirectRoomData } from '@/types/room'

export const roomApi = {
  /** 获取聊天室列表 */
  listRooms(params?: ListRoomsParams): Promise<ApiResponse<Room[]>> {
    return httpClient.get('/rooms', { params })
  },

  /** 获取我加入的聊天室 */
  getMyRooms(): Promise<Room[]> {
    return httpClient.get('/users/me/rooms')
  },

  /** 获取最近更新的聊天室 */
  listRecentRooms(params?: { limit?: number; offset?: number }): Promise<ApiResponse<Room[]>> {
    return httpClient.get('/rooms/recent', { params })
  },

  /** 获取聊天室详情 */
  getRoom(roomId: string): Promise<ApiResponse<Room>> {
    return httpClient.get(`/rooms/${roomId}`)
  },

  /** 创建聊天室 */
  createRoom(data: CreateRoomData): Promise<ApiResponse<Room>> {
    return httpClient.post('/rooms', data)
  },

  /** 更新聊天室 */
  updateRoom(roomId: string, data: UpdateRoomData): Promise<ApiResponse<Room>> {
    return httpClient.put(`/rooms/${roomId}`, data)
  },

  /** 删除聊天室 */
  deleteRoom(roomId: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/rooms/${roomId}`)
  },

  /** 加入聊天室 */
  joinRoom(roomId: string): Promise<ApiResponse<unknown>> {
    return httpClient.post(`/rooms/${roomId}/join`)
  },

  /** 离开聊天室 */
  leaveRoom(roomId: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/rooms/${roomId}/leave`)
  },

  /** 获取聊天室成员列表 */
  getRoomMembers(roomId: string): Promise<ApiResponse<RoomMember[]>> {
    return httpClient.get(`/rooms/${roomId}/members`)
  },

  /** 踢出成员 */
  kickMember(roomId: string, userId: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/rooms/${roomId}/members/${userId}`)
  },

  /** 设置成员角色 */
  setMemberRole(roomId: string, userId: string, role: 'admin' | 'member'): Promise<ApiResponse<unknown>> {
    return httpClient.put(`/rooms/${roomId}/members/${userId}/role`, { role })
  },

  /** 创建私聊房间 */
  createDirectRoom(data: CreateDirectRoomData): Promise<ApiResponse<DirectRoom>> {
    return httpClient.post('/rooms/direct', data)
  },

  /** 获取私聊房间列表 */
  getDirectRooms(): Promise<ApiResponse<DirectRoom[]>> {
    return httpClient.get('/rooms/direct/list')
  },
}
