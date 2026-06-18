export type UserStatus = 'online' | 'away' | 'busy' | 'offline'
export type UserRole = 'user' | 'admin' | 'super_admin'

export interface User {
  id: string
  username: string
  email: string
  avatar_url: string | null
  status: UserStatus
  is_active: boolean
  role: UserRole
  created_at: string
}

export interface UserInfo {
  id: string
  username: string
  avatar_url: string | null
}

export interface LoginCredentials {
  email: string
  password: string
}

export interface RegisterData {
  username: string
  email: string
  password: string
}

export interface AuthTokens {
  access_token: string
  refresh_token: string
}
