export interface RoomOwner {
  id: string
  username: string
  avatar_url: string | null
}

export interface Room {
  id: string
  name: string
  description: string | null
  owner: RoomOwner
  is_private: boolean
  max_members: number
  member_count: number
  created_at: string
  updated_at: string
  unread_count?: number
  last_message?: MessagePreview | null
}

export interface RoomMember {
  room_id: string
  user_id: string
  role: 'owner' | 'admin' | 'member'
  joined_at: string
  username: string
  email: string
  avatar_url: string | null
  user_status: 'online' | 'away' | 'busy' | 'offline'
}

export interface CreateRoomData {
  name: string
  description?: string
  is_private?: boolean
  max_members?: number
}

export interface UpdateRoomData {
  name?: string
  description?: string
  is_private?: boolean
  max_members?: number
}

export interface MessagePreview {
  id: string
  content: string
  sender_name: string
  created_at: string
}

export interface ListRoomsParams {
  search?: string
  limit?: number
  offset?: number
}
