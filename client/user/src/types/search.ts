/**
 * 搜索功能类型定义
 * 对应后端搜索相关接口
 */

import type { Room } from './room'

/** 搜索类型 */
export type SearchType = 'room' | 'user'

/** 搜索参数 */
export interface SearchParams {
  /** 搜索关键词 */
  keyword: string
  /** 每页数量 */
  limit?: number
  /** 偏移量 */
  offset?: number
}

/** 房间搜索结果 */
export interface RoomSearchResult {
  /** 房间列表 */
  rooms: Room[]
  /** 总数 */
  total: number
  /** 每页数量 */
  limit: number
  /** 当前偏移量 */
  offset: number
}

/** 用户搜索结果（简化版，用于搜索展示） */
export interface UserSearchItem {
  /** 用户ID */
  id: string
  /** 用户名 */
  username: string
  /** 头像URL */
  avatar_url: string | null
  /** 在线状态 */
  status: 'online' | 'offline' | 'away'
}

/** 用户搜索结果 */
export interface UserSearchResult {
  /** 用户列表 */
  users: UserSearchItem[]
  /** 总数 */
  total: number
}

/** 搜索状态 */
export interface SearchState {
  /** 当前搜索类型 */
  searchType: SearchType
  /** 当前关键词 */
  keyword: string
  /** 房间搜索结果 */
  roomResults: Room[]
  /** 用户搜索结果 */
  userResults: UserSearchItem[]
  /** 房间结果总数 */
  roomTotal: number
  /** 用户结果总数 */
  userTotal: number
  /** 加载状态 */
  loading: boolean
  /** 错误信息 */
  error: string | null
  /** 搜索历史 */
  history: string[]
}

/** 发现页面标签类型 */
export type DiscoverTabType = 'discover' | 'rooms' | 'users'

/** 公开房间发现参数 */
export interface DiscoverRoomsParams {
  /** 每页数量 */
  limit?: number
  /** 偏移量 */
  offset?: number
}
