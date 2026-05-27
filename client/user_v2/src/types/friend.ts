import type { UserStatus } from '@/types/user'

/** 好友请求状态 */
export type FriendRequestStatus = 'pending' | 'accepted' | 'rejected'

/** 好友请求操作 */
export type FriendRequestAction = 'accept' | 'reject'

/** 好友信息 */
export interface Friend {
  id: string
  friend: {
    id: string
    username: string
    avatar_url: string | null
    status: UserStatus
  }
  created_at: string
  last_online?: string
}

/** 好友请求 */
export interface FriendRequest {
  id: string
  sender: {
    id: string
    username: string
    avatar_url: string | null
  }
  receiver: {
    id: string
    username: string
    avatar_url: string | null
  }
  status: FriendRequestStatus
  message?: string
  created_at: string
  updated_at: string
}

/** 发送好友请求数据 */
export interface SendFriendRequestData {
  target_user_id: string
  message?: string
}

/** 好友列表响应 */
export interface FriendListResponse {
  friends: Friend[]
  total: number
}

/** 好友请求列表响应 */
export interface FriendRequestListResponse {
  requests: FriendRequest[]
  total: number
}
