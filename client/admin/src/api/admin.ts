import { http } from './request'
import type { ApiResponse, UserInfo, UserRole, UserStatus } from '@/types'

// ==================== 用户管理类型 ====================

/**
 * 用户列表查询参数
 */
export interface UserListParams {
  /** 页码 */
  page?: number
  /** 每页数量 */
  page_size?: number
  /** 搜索关键词 */
  search?: string
}

/**
 * 用户列表响应数据
 */
export interface UserListData {
  /** 用户列表 */
  users: UserInfo[]
  /** 总用户数 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页数量 */
  page_size: number
}

/**
 * 修改用户角色请求
 */
export interface UpdateUserRoleRequest {
  /** 新角色 */
  role: UserRole
}

/**
 * 修改用户状态请求
 */
export interface UpdateUserStatusRequest {
  /** 新状态 */
  status: UserStatus
}

/**
 * 重置密码请求
 */
export interface ResetPasswordRequest {
  /** 新密码 */
  password: string
}

// ==================== 消息管理类型 ====================

/**
 * 管理员消息信息
 */
export interface AdminMessageInfo {
  /** 消息ID */
  id: string
  /** 聊天室ID */
  room_id: string
  /** 发送者信息 */
  sender: {
    id: string
    username: string
    avatar_url: string | null
  }
  /** 消息内容 */
  content: string
  /** 消息类型 */
  message_type: 'text' | 'image' | 'file' | 'system'
  /** 回复的消息ID */
  reply_to: string | null
  /** 是否已删除 */
  is_deleted: boolean
  /** 创建时间 */
  created_at: string
  /** 编辑次数 */
  edit_count: number
  /** 最后编辑时间 */
  edited_at: string | null
}

/**
 * 管理员消息列表查询参数
 */
export interface AdminMessageListParams {
  /** 页码 */
  page?: number
  /** 每页数量 */
  page_size?: number
  /** 搜索关键词 */
  search?: string
  /** 过滤特定房间ID */
  room_id?: string
}

/**
 * 管理员消息列表响应数据（对象格式）
 * 用于 GET /admin/messages
 */
export interface AdminMessageListData {
  /** 消息列表 */
  messages: AdminMessageInfo[]
  /** 总消息数 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页数量 */
  page_size: number
}

/**
 * 房间消息列表响应数据（数组格式）
 * 用于 GET /admin/rooms/:room_id/messages
 */
export type AdminRoomMessageListData = AdminMessageInfo[]

/**
 * 管理员相关 API
 */
export const adminApi = {
  /**
   * 获取用户列表
   * @param params 查询参数
   * @returns 用户列表数据
   */
  getUserList(params: UserListParams = {}): Promise<ApiResponse<UserListData>> {
    const queryParams = new URLSearchParams()
    if (params.page) queryParams.append('page', String(params.page))
    if (params.page_size) queryParams.append('page_size', String(params.page_size))
    if (params.search) queryParams.append('search', params.search)

    const query = queryParams.toString()
    return http.get<UserListData>(`/admin/users${query ? `?${query}` : ''}`)
  },

  /**
   * 获取用户详情
   * @param userId 用户ID
   * @returns 用户详情
   */
  getUserDetail(userId: string): Promise<ApiResponse<UserInfo>> {
    return http.get<UserInfo>(`/admin/users/${userId}`)
  },

  /**
   * 修改用户角色
   * @param userId 用户ID
   * @param data 角色数据
   * @returns 更新后的用户信息
   */
  updateUserRole(userId: string, data: UpdateUserRoleRequest): Promise<ApiResponse<UserInfo>> {
    return http.put<UserInfo>(`/admin/users/${userId}/role`, data)
  },

  /**
   * 修改用户状态（禁用/启用）
   * @param userId 用户ID
   * @param data 状态数据
   * @returns 更新后的用户信息
   */
  updateUserStatus(userId: string, data: UpdateUserStatusRequest): Promise<ApiResponse<UserInfo>> {
    return http.put<UserInfo>(`/admin/users/${userId}/status`, data)
  },

  /**
   * 删除用户
   * @param userId 用户ID
   * @returns 操作结果
   */
  deleteUser(userId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/admin/users/${userId}`)
  },

  /**
   * 重置用户密码
   * @param userId 用户ID
   * @param data 密码数据
   * @returns 操作结果
   */
  resetUserPassword(userId: string, data: ResetPasswordRequest): Promise<ApiResponse<void>> {
    return http.put<void>(`/admin/users/${userId}/password`, data)
  },

  // ==================== 消息管理 API ====================

  /**
   * 获取消息列表（管理员）
   * @param params 查询参数
   * @returns 消息列表数据
   */
  getMessageList(params: AdminMessageListParams = {}): Promise<ApiResponse<AdminMessageListData>> {
    const queryParams = new URLSearchParams()
    if (params.page) queryParams.append('page', String(params.page))
    if (params.page_size) queryParams.append('page_size', String(params.page_size))
    if (params.search) queryParams.append('search', params.search)
    if (params.room_id) queryParams.append('room_id', params.room_id)

    const query = queryParams.toString()
    return http.get<AdminMessageListData>(`/admin/messages${query ? `?${query}` : ''}`)
  },

  /**
   * 获取房间消息记录（管理员）
   * @param roomId 房间ID
   * @param params 查询参数
   * @returns 消息列表数据（数组格式）
   */
  getRoomMessages(roomId: string, params: Omit<AdminMessageListParams, 'room_id'> = {}): Promise<ApiResponse<AdminRoomMessageListData>> {
    const queryParams = new URLSearchParams()
    if (params.page) queryParams.append('page', String(params.page))
    if (params.page_size) queryParams.append('page_size', String(params.page_size))
    if (params.search) queryParams.append('search', params.search)

    const query = queryParams.toString()
    return http.get<AdminRoomMessageListData>(`/admin/rooms/${roomId}/messages${query ? `?${query}` : ''}`)
  },

  /**
   * 删除违规消息（管理员）
   * @param messageId 消息ID
   * @returns 操作结果
   */
  deleteMessage(messageId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/admin/messages/${messageId}`)
  },

  /**
   * 踢出房间成员（管理员）
   * @param roomId 房间ID
   * @param userId 用户ID
   * @returns 操作结果
   */
  kickRoomMember(roomId: string, userId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/admin/rooms/${roomId}/members/${userId}`)
  },

  /**
   * 设置房间成员角色（管理员）
   * @param roomId 房间ID
   * @param userId 用户ID
   * @param role 角色（owner/admin/member）
   * @returns 操作结果
   */
  setRoomMemberRole(roomId: string, userId: string, role: 'owner' | 'admin' | 'member'): Promise<ApiResponse<void>> {
    return http.put<void>(`/admin/rooms/${roomId}/members/${userId}/role`, { role })
  },
}
