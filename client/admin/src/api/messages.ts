import { http } from './request'
import type { ApiResponse } from '@/types'

/**
 * 发送者信息
 */
export interface SenderInfo {
  /** 用户ID */
  id: string
  /** 用户名 */
  username: string
  /** 头像URL */
  avatar_url: string | null
}

/**
 * 被回复消息信息
 */
export interface ReplyToMessage {
  /** 消息ID */
  id: string
  /** 发送者信息 */
  sender: SenderInfo
  /** 消息内容 */
  content: string
  /** 发送时间 */
  created_at: string
}

/**
 * 消息信息
 */
export interface MessageInfo {
  /** 消息ID */
  id: string
  /** 聊天室ID */
  room_id: string
  /** 发送者信息 */
  sender: SenderInfo
  /** 消息内容 */
  content: string
  /** 消息类型 */
  message_type: 'text' | 'image' | 'file' | 'system'
  /** 回复的消息ID */
  reply_to: string | null
  /** 被回复消息的详细信息 */
  reply_to_message: ReplyToMessage | null
  /** 是否已删除 */
  is_deleted: boolean
  /** 发送时间 */
  created_at: string
  /** 编辑次数 */
  edit_count: number
  /** 最后编辑时间 */
  edited_at: string | null
}

/**
 * 编辑历史记录
 */
export interface EditHistory {
  /** 历史记录ID */
  id: string
  /** 消息ID */
  message_id: string
  /** 编辑前的内容 */
  old_content: string
  /** 编辑后的内容 */
  new_content: string
  /** 编辑者ID */
  edited_by: string
  /** 编辑时间 */
  edited_at: string
}

/**
 * 房间消息历史响应
 */
export interface RoomMessagesResponse {
  /** 消息列表 */
  messages: MessageInfo[]
  /** 本次返回的消息数量 */
  total: number
  /** 是否还有更多历史消息 */
  has_more: boolean
}

/**
 * 获取房间消息历史参数
 */
export interface GetRoomMessagesParams {
  /** 每页数量 */
  limit?: number
  /** 游标，获取此消息ID之前的历史消息 */
  before?: string
}

/**
 * 搜索消息参数
 */
export interface SearchMessagesParams {
  /** 搜索关键词 */
  q: string
  /** 限定聊天室ID */
  room_id?: string
  /** 结果数量限制 */
  limit?: number
}

/**
 * 编辑消息请求
 */
export interface EditMessageRequest {
  /** 新的消息内容 */
  content: string
}

/**
 * 消息相关 API
 */
export const messagesApi = {
  /**
   * 获取房间消息历史
   * @param roomId 聊天室ID
   * @param params 查询参数
   * @returns 消息历史
   */
  getRoomMessages(roomId: string, params: GetRoomMessagesParams = {}): Promise<ApiResponse<RoomMessagesResponse>> {
    const queryParams = new URLSearchParams()
    if (params.limit) queryParams.append('limit', String(params.limit))
    if (params.before) queryParams.append('before', params.before)

    const query = queryParams.toString()
    return http.get<RoomMessagesResponse>(`/rooms/${roomId}/messages${query ? `?${query}` : ''}`)
  },

  /**
   * 搜索消息
   * @param params 搜索参数
   * @returns 消息列表
   */
  searchMessages(params: SearchMessagesParams): Promise<ApiResponse<MessageInfo[]>> {
    const queryParams = new URLSearchParams()
    queryParams.append('q', params.q)
    if (params.room_id) queryParams.append('room_id', params.room_id)
    if (params.limit) queryParams.append('limit', String(params.limit))

    const query = queryParams.toString()
    return http.get<MessageInfo[]>(`/messages/search?${query}`)
  },

  /**
   * 编辑消息
   * @param messageId 消息ID
   * @param data 编辑请求数据
   * @returns 更新后的消息
   */
  editMessage(messageId: string, data: EditMessageRequest): Promise<ApiResponse<MessageInfo>> {
    return http.put<MessageInfo>(`/messages/${messageId}`, data)
  },

  /**
   * 删除消息
   * @param messageId 消息ID
   * @returns 操作结果
   */
  deleteMessage(messageId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/messages/${messageId}`)
  },

  /**
   * 获取消息编辑历史
   * @param messageId 消息ID
   * @param limit 限制数量
   * @returns 编辑历史列表
   */
  getMessageHistory(messageId: string, limit?: number): Promise<ApiResponse<EditHistory[]>> {
    const queryParams = new URLSearchParams()
    if (limit) queryParams.append('limit', String(limit))

    const query = queryParams.toString()
    return http.get<EditHistory[]>(`/messages/${messageId}/history${query ? `?${query}` : ''}`)
  },
}
