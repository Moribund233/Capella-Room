/**
 * 消息管理 API
 * 负责消息的发送、查询、搜索等操作
 */

import { apiClient } from './client'
import type { Message } from '@/types/api'

export type { Message } from '@/types/api'

// 消息列表响应
export interface MessageListResponse {
  messages: Message[]
  total: number
  has_more: boolean
}

// 发送消息请求
export interface SendMessageRequest {
  content: string
  room_id: string
  type?: 'text' | 'image' | 'file'
  reply_to?: string
}

// 搜索消息参数
export interface SearchMessageParams {
  query?: string
  room_id?: string
  sender_id?: string
  start_time?: string
  end_time?: string
  page?: number
  per_page?: number
}

/**
 * 获取房间消息列表
 * @param roomId 房间ID
 * @param params 查询参数
 * @returns 消息列表
 */
export async function getRoomMessages(
  roomId: string,
  params?: {
    before?: string
    after?: string
    limit?: number
  }
): Promise<MessageListResponse> {
  const queryParams: Record<string, string> = {}
  if (params?.before) queryParams.before = params.before
  if (params?.after) queryParams.after = params.after
  if (params?.limit) queryParams.limit = String(params.limit)

  const response = await apiClient.get<MessageListResponse>(`/api/v1/rooms/${roomId}/messages`, queryParams)
  const data = response.data
  // 确保返回的数据格式正确
  if (!data) {
    return {
      messages: [],
      total: 0,
      has_more: false,
    }
  }
  return {
    messages: data.messages || [],
    total: data.total || 0,
    has_more: data.has_more || false,
  }
}

/**
 * 发送消息
 * @param data 消息数据
 * @returns 发送的消息
 */
export async function sendMessage(data: SendMessageRequest): Promise<Message> {
  const response = await apiClient.post<Message>('/api/v1/messages', data)
  return response.data
}

/**
 * 搜索消息
 * @param params 搜索参数
 * @returns 搜索结果
 */
export async function searchMessages(params: SearchMessageParams): Promise<MessageListResponse> {
  const queryParams: Record<string, string> = {}
  if (params.query) queryParams.q = params.query
  if (params.room_id) queryParams.room_id = params.room_id
  if (params.sender_id) queryParams.sender_id = params.sender_id
  if (params.start_time) queryParams.start_time = params.start_time
  if (params.end_time) queryParams.end_time = params.end_time
  if (params.page) queryParams.page = String(params.page)
  if (params.per_page) queryParams.per_page = String(params.per_page)

  const response = await apiClient.get<MessageListResponse>('/api/v1/messages/search', queryParams)
  const data = response.data
  // 确保返回的数据格式正确
  if (!data) {
    return {
      messages: [],
      total: 0,
      has_more: false,
    }
  }
  return {
    messages: data.messages || [],
    total: data.total || 0,
    has_more: data.has_more || false,
  }
}

/**
 * 删除消息
 * @param messageId 消息ID
 */
export async function deleteMessage(messageId: string): Promise<void> {
  await apiClient.delete(`/api/v1/messages/${messageId}`)
}

/**
 * 编辑消息
 * @param messageId 消息ID
 * @param content 新内容
 * @returns 更新后的消息
 */
export async function editMessage(messageId: string, content: string): Promise<Message> {
  const response = await apiClient.patch<Message>(`/api/v1/messages/${messageId}`, { content })
  return response.data
}

/**
 * 获取消息详情
 * @param messageId 消息ID
 * @returns 消息详情
 */
export async function getMessageDetail(messageId: string): Promise<Message> {
  const response = await apiClient.get<Message>(`/api/v1/messages/${messageId}`)
  return response.data
}
