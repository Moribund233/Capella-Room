/**
 * 房间相关 API
 */

import { apiClient } from './client'
import type { Room, Message, MessageListResponse, PaginationParams } from '@/types/api'

// 房间列表响应（兼容直接返回数组或包装对象）
export interface RoomListResponse {
  items: Room[]
  total: number
  page: number
  per_page: number
}

/**
 * 获取房间列表
 */
export async function getRooms(params?: PaginationParams): Promise<RoomListResponse> {
  const queryParams: Record<string, string> = {}
  if (params?.page) queryParams.page = params.page.toString()
  if (params?.per_page) queryParams.per_page = params.per_page.toString()
  if (params?.page_size) queryParams.page_size = params.page_size.toString()

  const response = await apiClient.get<Room[] | RoomListResponse>('/rooms', queryParams)
  const data = response.data

  // 适配两种可能的响应格式：直接返回数组或包装在 items 字段中
  if (Array.isArray(data)) {
    return {
      items: data,
      total: data.length,
      page: 1,
      per_page: data.length,
    }
  }

  return {
    items: data.items || [],
    total: data.total || 0,
    page: data.page || 1,
    per_page: data.per_page || 20,
  }
}

/**
 * 获取房间详情
 */
export async function getRoom(roomId: string): Promise<Room> {
  const response = await apiClient.get<Room>(`/rooms/${roomId}`)
  return response.data
}

/**
 * 创建房间
 */
export async function createRoom(data: {
  name: string
  description?: string
  is_private?: boolean
  max_members?: number
}): Promise<Room> {
  const response = await apiClient.post<Room>('/rooms', data)
  return response.data
}

/**
 * 更新房间
 */
export async function updateRoom(
  roomId: string,
  data: {
    name?: string
    description?: string
    is_private?: boolean
    max_members?: number
  }
): Promise<Room> {
  const response = await apiClient.put<Room>(`/rooms/${roomId}`, data)
  return response.data
}

/**
 * 删除房间
 */
export async function deleteRoom(roomId: string): Promise<void> {
  await apiClient.delete<void>(`/rooms/${roomId}`)
}

/**
 * 获取房间消息
 */
export async function getRoomMessages(
  roomId: string,
  params?: PaginationParams
): Promise<MessageListResponse> {
  const queryParams: Record<string, string> = {}
  if (params?.page) queryParams.page = params.page.toString()
  if (params?.per_page) queryParams.per_page = params.per_page.toString()
  if (params?.page_size) queryParams.page_size = params.page_size.toString()

  const response = await apiClient.get<MessageListResponse>(
    `/rooms/${roomId}/messages`,
    queryParams
  )
  return response.data
}

/**
 * 发送消息到房间
 */
export async function sendMessage(
  roomId: string,
  content: string,
  type: 'text' | 'image' | 'file' = 'text',
  replyTo?: string
): Promise<Message> {
  const response = await apiClient.post<Message>(`/rooms/${roomId}/messages`, {
    content,
    type,
    reply_to: replyTo,
  })
  return response.data
}

/**
 * 加入房间
 */
export async function joinRoom(roomId: string): Promise<void> {
  await apiClient.post<void>(`/rooms/${roomId}/join`)
}

/**
 * 离开房间
 */
export async function leaveRoom(roomId: string): Promise<void> {
  await apiClient.post<void>(`/rooms/${roomId}/leave`)
}
