/**
 * 房间管理 API
 * 负责房间的创建、查询、更新、删除等操作
 */

import { apiClient } from './client'
import type { Room } from '@/types/api'

// 房间列表响应
export interface RoomListResponse {
  rooms: Room[]
  total: number
}

// 创建房间请求
export interface CreateRoomRequest {
  name: string
  description?: string
  is_private: boolean
}

// 更新房间请求
export interface UpdateRoomRequest {
  name?: string
  description?: string
  is_private?: boolean
}

// 房间成员信息
export interface RoomMember {
  id: string
  username: string
  role: 'owner' | 'admin' | 'member'
  joined_at: string
}

// 房间详情响应
export interface RoomDetailResponse {
  room: Room
  members: RoomMember[]
}

/**
 * 获取房间列表
 * @param params 查询参数
 * @returns 房间列表
 */
export async function getRooms(params?: {
  search?: string
  is_public?: boolean
  page?: number
  per_page?: number
}): Promise<Room[]> {
  // 转换参数为字符串格式
  const queryParams: Record<string, string> = {}
  if (params?.search) queryParams.search = params.search
  if (params?.is_public !== undefined) queryParams.is_public = String(params.is_public)
  if (params?.page) queryParams.page = String(params.page)
  if (params?.per_page) queryParams.per_page = String(params.per_page)

  const response = await apiClient.get<Room[] | RoomListResponse>('/api/v1/rooms', queryParams)
  // 适配两种可能的响应格式：直接返回数组或包装在 rooms 字段中
  const data = response.data
  if (Array.isArray(data)) {
    return data
  }
  return data.rooms || []
}

/**
 * 获取房间详情
 * @param roomId 房间ID
 * @returns 房间详情
 */
export async function getRoomDetail(roomId: string): Promise<RoomDetailResponse> {
  const response = await apiClient.get<RoomDetailResponse>(`/api/v1/rooms/${roomId}`)
  return response.data
}

/**
 * 创建房间
 * @param data 房间信息
 * @returns 创建的房间
 */
export async function createRoom(data: CreateRoomRequest): Promise<Room> {
  const response = await apiClient.post<{ room: Room }>('/api/v1/rooms', data)
  return response.data.room
}

/**
 * 更新房间
 * @param roomId 房间ID
 * @param data 更新信息
 * @returns 更新后的房间
 */
export async function updateRoom(roomId: string, data: UpdateRoomRequest): Promise<Room> {
  const response = await apiClient.put<{ room: Room }>(`/api/v1/rooms/${roomId}`, data)
  return response.data.room
}

/**
 * 删除房间
 * @param roomId 房间ID
 */
export async function deleteRoom(roomId: string): Promise<void> {
  await apiClient.delete(`/api/v1/rooms/${roomId}`)
}

/**
 * 加入房间
 * @param roomId 房间ID
 */
export async function joinRoom(roomId: string): Promise<void> {
  await apiClient.post(`/api/v1/rooms/${roomId}/join`, {})
}

/**
 * 离开房间
 * @param roomId 房间ID
 */
export async function leaveRoom(roomId: string): Promise<void> {
  await apiClient.delete(`/api/v1/rooms/${roomId}/leave`)
}

/**
 * 获取用户加入的房间列表
 * @returns 房间列表
 */
export async function getMyRooms(): Promise<Room[]> {
  const response = await apiClient.get<Room[]>('/api/v1/users/me/rooms')
  return response.data
}
