import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type {
  FriendRequest,
  SendFriendRequestData,
  FriendListResponse,
  FriendRequestListResponse,
} from '@/types/friend'

export const friendApi = {
  /** 获取好友列表 */
  getFriends(): Promise<ApiResponse<FriendListResponse>> {
    return httpClient.get('/users/friends')
  },

  /** 发送好友请求 */
  sendFriendRequest(data: SendFriendRequestData): Promise<ApiResponse<FriendRequest>> {
    return httpClient.post('/users/friends/requests', data)
  },

  /** 获取收到的请求 */
  getReceivedRequests(): Promise<ApiResponse<FriendRequestListResponse>> {
    return httpClient.get('/users/friends/requests/received')
  },

  /** 获取发送的请求 */
  getSentRequests(): Promise<ApiResponse<FriendRequestListResponse>> {
    return httpClient.get('/users/friends/requests/sent')
  },

  /** 处理好友请求 */
  handleFriendRequest(requestId: string, accept: boolean): Promise<ApiResponse<FriendRequest>> {
    return httpClient.post('/users/friends/requests/handle', { request_id: requestId, accept })
  },

  /** 取消好友请求 */
  cancelFriendRequest(requestId: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/users/friends/requests/${requestId}`)
  },

  /** 删除好友 */
  deleteFriend(friendId: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/users/friends/${friendId}`)
  },
}
