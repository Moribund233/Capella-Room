import { http } from './request'
import type { ApiResponse } from '@/types'

/**
 * 聊天室信息
 */
export interface RoomInfo {
  /** 聊天室ID */
  id: string
  /** 聊天室名称 */
  name: string
  /** 聊天室描述 */
  description: string | null
  /** 房主信息 */
  owner: {
    id: string
    username: string
    avatar_url: string | null
  }
  /** 是否为私有聊天室 */
  is_private: boolean
  /** 最大成员数 */
  max_members: number
  /** 当前成员数量 */
  member_count: number
  /** 创建时间 */
  created_at: string
  /** 更新时间 */
  updated_at: string
}

/**
 * 成员信息
 */
export interface MemberInfo {
  /** 房间ID */
  room_id: string
  /** 用户ID */
  id: string
  /** 用户ID（与id相同，后端返回字段） */
  user_id: string
  /** 用户名 */
  username: string
  /** 头像URL */
  avatar_url: string | null
  /** 角色 */
  role: 'owner' | 'admin' | 'member'
  /** 加入时间 */
  joined_at: string
  /** 用户在线状态 */
  user_status: 'online' | 'offline' | 'away'
  /** 用户邮箱 */
  email: string
}

/**
 * 聊天室列表查询参数
 */
export interface RoomListParams {
  /** 搜索关键词 */
  search?: string
  /** 每页数量 */
  limit?: number
  /** 偏移量 */
  offset?: number
}

/**
 * 创建聊天室请求
 */
export interface CreateRoomRequest {
  /** 聊天室名称 */
  name: string
  /** 聊天室描述 */
  description?: string
  /** 是否为私有聊天室 */
  is_private: boolean
  /** 最大成员数 */
  max_members?: number
}

/**
 * 更新聊天室请求
 */
export interface UpdateRoomRequest {
  /** 聊天室名称 */
  name?: string
  /** 聊天室描述 */
  description?: string
  /** 是否为私有聊天室 */
  is_private?: boolean
  /** 最大成员数 */
  max_members?: number
}

/**
 * 设置成员角色请求
 */
export interface SetMemberRoleRequest {
  /** 角色 */
  role: 'admin' | 'member' | 'owner'
}

/**
 * 聊天室相关 API
 */
export const roomsApi = {
  /**
   * 获取聊天室列表
   * @param params 查询参数
   * @returns 聊天室列表
   */
  getRoomList(params: RoomListParams = {}): Promise<ApiResponse<RoomInfo[]>> {
    const queryParams = new URLSearchParams()
    if (params.search) queryParams.append('search', params.search)
    if (params.limit) queryParams.append('limit', String(params.limit))
    if (params.offset) queryParams.append('offset', String(params.offset))

    const query = queryParams.toString()
    return http.get<RoomInfo[]>(`/rooms${query ? `?${query}` : ''}`)
  },

  /**
   * 获取最近更新的聊天室
   * @param params 查询参数
   * @returns 聊天室列表
   */
  getRecentRooms(params: Omit<RoomListParams, 'search'> = {}): Promise<ApiResponse<RoomInfo[]>> {
    const queryParams = new URLSearchParams()
    if (params.limit) queryParams.append('limit', String(params.limit))
    if (params.offset) queryParams.append('offset', String(params.offset))

    const query = queryParams.toString()
    return http.get<RoomInfo[]>(`/rooms/recent${query ? `?${query}` : ''}`)
  },

  /**
   * 获取聊天室详情
   * @param roomId 聊天室ID
   * @returns 聊天室详情
   */
  getRoomDetail(roomId: string): Promise<ApiResponse<RoomInfo>> {
    return http.get<RoomInfo>(`/rooms/${roomId}`)
  },

  /**
   * 创建聊天室
   * @param data 创建请求数据
   * @returns 创建的聊天室信息
   */
  createRoom(data: CreateRoomRequest): Promise<ApiResponse<RoomInfo>> {
    return http.post<RoomInfo>('/rooms', data)
  },

  /**
   * 更新聊天室信息
   * @param roomId 聊天室ID
   * @param data 更新请求数据
   * @returns 更新后的聊天室信息
   */
  updateRoom(roomId: string, data: UpdateRoomRequest): Promise<ApiResponse<RoomInfo>> {
    return http.put<RoomInfo>(`/rooms/${roomId}`, data)
  },

  /**
   * 删除聊天室
   * @param roomId 聊天室ID
   * @returns 操作结果
   */
  deleteRoom(roomId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/rooms/${roomId}`)
  },

  /**
   * 加入聊天室
   * @param roomId 聊天室ID
   * @returns 操作结果
   */
  joinRoom(roomId: string): Promise<ApiResponse<void>> {
    return http.post<void>(`/rooms/${roomId}/join`)
  },

  /**
   * 离开聊天室
   * @param roomId 聊天室ID
   * @returns 操作结果
   */
  leaveRoom(roomId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/rooms/${roomId}/leave`)
  },

  /**
   * 获取成员列表
   * @param roomId 聊天室ID
   * @returns 成员列表
   */
  getMembers(roomId: string): Promise<ApiResponse<MemberInfo[]>> {
    return http.get<MemberInfo[]>(`/rooms/${roomId}/members`)
  },

  /**
   * 踢出成员
   * @param roomId 聊天室ID
   * @param userId 用户ID
   * @returns 操作结果
   */
  kickMember(roomId: string, userId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/rooms/${roomId}/members/${userId}`)
  },

  /**
   * 设置成员角色
   * @param roomId 聊天室ID
   * @param userId 用户ID
   * @param data 角色数据
   * @returns 操作结果
   */
  setMemberRole(roomId: string, userId: string, data: SetMemberRoleRequest): Promise<ApiResponse<void>> {
    return http.put<void>(`/rooms/${roomId}/members/${userId}/role`, data)
  },
}
