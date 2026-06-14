import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { Message, MessageEditResponse, PinnedMessage } from '@/types/message'

export interface MessageListResponse {
  messages: Message[]
  total: number
  has_more: boolean
}

export interface GetMessagesParams {
  limit?: number
  before?: string
}

export interface SearchMessagesParams {
  q: string
  room_id?: string
  limit?: number
}

export const messageApi = {
  /** 获取房间消息历史（游标分页） */
  getRoomMessages(
    roomId: string,
    params?: GetMessagesParams,
  ): Promise<ApiResponse<MessageListResponse>> {
    return httpClient.get(`/rooms/${roomId}/messages`, { params })
  },

  /** 搜索消息 */
  searchMessages(params: SearchMessagesParams): Promise<ApiResponse<Message[]>> {
    return httpClient.get('/messages/search', { params })
  },

  /** 获取消息编辑历史 */
  getEditHistory(messageId: string): Promise<MessageEditResponse[]> {
    return httpClient.get(`/messages/${messageId}/history`)
  },

  /** 获取房间置顶消息 */
  getRoomPinnedMessages(roomId: string): Promise<ApiResponse<PinnedMessage[]>> {
    return httpClient.get(`/rooms/${roomId}/pinned-messages`)
  },

  /** 置顶消息 */
  pinMessage(messageId: string): Promise<ApiResponse<void>> {
    return httpClient.post(`/messages/${messageId}/pin`)
  },

  /** 取消置顶消息 */
  unpinMessage(messageId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/messages/${messageId}/pin`)
  },
}
