import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type {
  SearchParams,
  RoomSearchResult,
  UserSearchResult,
  DiscoverRoomsParams,
} from '@/types/search'
import type { Room } from '@/types/room'

/**
 * 搜索相关 API
 */
export const searchApi = {
  /**
   * 搜索房间
   * @param params 搜索参数
   * @returns 房间搜索结果
   */
  searchRooms(params: SearchParams): Promise<ApiResponse<RoomSearchResult>> {
    const { keyword, limit = 20, offset = 0 } = params
    return httpClient.get('/rooms', {
      params: {
        search: keyword,
        limit,
        offset,
      },
    })
  },

  /**
   * 搜索用户
   * @param params 搜索参数
   * @returns 用户搜索结果
   */
  searchUsers(params: SearchParams): Promise<ApiResponse<UserSearchResult>> {
    const { keyword, limit = 20, offset = 0 } = params
    return httpClient.get('/users/search', {
      params: {
        keyword,
        limit,
        offset,
      },
    })
  },

  /**
   * 获取公开房间列表（发现页推荐）
   * @param params 分页参数
   * @returns 公开房间列表
   */
  getPublicRooms(
    params?: DiscoverRoomsParams,
  ): Promise<ApiResponse<Room[]>> {
    const { limit = 20, offset = 0 } = params || {}
    return httpClient.get('/rooms', {
      params: {
        limit,
        offset,
      },
    })
  },

  /**
   * 获取最近活跃的公开房间
   * @param params 分页参数
   * @returns 最近活跃的房间列表
   */
  getRecentPublicRooms(
    params?: DiscoverRoomsParams,
  ): Promise<ApiResponse<Room[]>> {
    const { limit = 20, offset = 0 } = params || {}
    return httpClient.get('/rooms/recent', {
      params: {
        limit,
        offset,
      },
    })
  },
}
