# API 集成规范

本文档定义了与后端 API 集成的规范，包括 HTTP API 和 WebSocket 的使用方式。

## 基础配置

### API 基础 URL

```typescript
// .env.development
VITE_API_BASE_URL=http://localhost:8080/api/v1
VITE_WS_URL=ws://localhost:8080/ws

// .env.production
VITE_API_BASE_URL=/api/v1
VITE_WS_URL=/ws
```

### HTTP 客户端配置

```typescript
// src/services/http.ts
import axios from 'axios'

const httpClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器
httpClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('access_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => Promise.reject(error)
)

// 响应拦截器
httpClient.interceptors.response.use(
  (response) => response.data,
  async (error) => {
    const originalRequest = error.config
    
    // Token 过期，尝试刷新
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true
      try {
        await refreshToken()
        return httpClient(originalRequest)
      } catch {
        // 刷新失败，跳转登录
        window.location.href = '/login'
        return Promise.reject(error)
      }
    }
    
    return Promise.reject(error)
  }
)
```

## 类型定义

### 基础响应类型

```typescript
// src/types/api.ts

// API 统一响应格式
export interface ApiResponse<T> {
  success: boolean
  data?: T
  code?: string
  error?: string
  message?: string
}

// 分页响应
export interface PaginatedResponse<T> {
  data: T[]
  total: number
  has_more: boolean
}

// 分页请求参数
export interface PaginationParams {
  limit?: number
  offset?: number
  before?: string  // 游标分页
}
```

### 用户相关类型

```typescript
// src/types/user.ts

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
```

### 聊天室相关类型

```typescript
// src/types/room.ts

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
  unread_count?: number  // 前端添加的字段
  last_message?: MessagePreview
}

export interface RoomMember {
  id: string
  username: string
  avatar_url: string | null
  role: 'owner' | 'admin' | 'member'
  joined_at: string
}

export interface CreateRoomData {
  name: string
  description?: string
  is_private?: boolean
  max_members?: number
}
```

### 消息相关类型

```typescript
// src/types/message.ts

export type MessageType = 'text' | 'image' | 'file'

export interface MessageSender {
  id: string
  username: string
  avatar_url: string | null
}

export interface ReplyToMessage {
  id: string
  sender: MessageSender
  content: string
  created_at: string
}

export interface Message {
  id: string
  room_id: string
  sender: MessageSender
  content: string
  message_type: MessageType
  reply_to: string | null
  reply_to_message: ReplyToMessage | null
  is_deleted: boolean
  created_at: string
  edit_count: number
  edited_at: string | null
  
  // 前端状态
  sending?: boolean
  error?: boolean
}

export interface MessagePreview {
  id: string
  content: string
  sender_name: string
  created_at: string
}
```

### WebSocket 消息类型

```typescript
// src/types/websocket.ts

// WebSocket 消息基类
export interface WebSocketMessage {
  type: string
  payload: unknown
}

// 客户端发送的消息
export interface ChatMessagePayload {
  room_id: string
  content: string
  reply_to?: string | null
}

export interface JoinRoomPayload {
  room_id: string
}

export interface UpdateStatusPayload {
  status: 'online' | 'away' | 'busy' | 'offline'
}

// 服务端发送的消息
export interface NewMessagePayload {
  message_id: string
  room_id: string
  sender_id: string
  sender_name: string
  content: string
  reply_to: string | null
  reply_to_message: ReplyToMessage | null
  created_at: string
}

export interface UserStatusChangedPayload {
  user_id: string
  username: string
  status: UserStatus
}

export interface OnlineUsersPayload {
  room_id: string
  users: Array<{
    id: string
    username: string
    avatar_url: string | null
    status: UserStatus
  }>
}

export interface TypingPayload {
  room_id: string
  user_id: string
  username: string
}

// 消息类型枚举
export enum WebSocketMessageType {
  // 客户端发送
  CHAT_MESSAGE = 'ChatMessage',
  TYPING = 'Typing',
  STOP_TYPING = 'StopTyping',
  JOIN_ROOM = 'JoinRoom',
  LEAVE_ROOM = 'LeaveRoom',
  UPDATE_STATUS = 'UpdateStatus',
  MESSAGE_READ = 'MessageRead',
  EDIT_MESSAGE = 'EditMessage',
  DELETE_MESSAGE = 'DeleteMessage',
  GET_MISSED_MESSAGES = 'GetMissedMessages',
  
  // 服务端发送
  NEW_MESSAGE = 'NewMessage',
  USER_TYPING = 'UserTyping',
  USER_STOP_TYPING = 'UserStopTyping',
  ROOM_JOINED = 'RoomJoined',
  ROOM_LEFT = 'RoomLeft',
  USER_JOINED = 'UserJoined',
  USER_LEFT = 'UserLeft',
  ONLINE_USERS = 'OnlineUsers',
  USER_STATUS_CHANGED = 'UserStatusChanged',
  MESSAGE_READ_RECEIPT = 'MessageReadReceipt',
  MESSAGE_EDITED = 'MessageEdited',
  MESSAGE_DELETED = 'MessageDeleted',
  MISSED_MESSAGES = 'MissedMessages',
  MENTIONED = 'Mentioned',
  ERROR = 'Error'
}
```

## API 模块封装

### 认证 API

```typescript
// src/api/auth.ts
import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { LoginCredentials, RegisterData, User, AuthTokens } from '@/types/user'

export const authApi = {
  /**
   * 用户登录
   * @param credentials 登录凭证
   * @returns 用户信息
   */
  login(credentials: LoginCredentials): Promise<ApiResponse<User>> {
    return httpClient.post('/auth/login', credentials)
  },

  /**
   * 用户注册
   * @param data 注册信息
   * @returns 新用户信息
   */
  register(data: RegisterData): Promise<ApiResponse<User>> {
    return httpClient.post('/auth/register', data)
  },

  /**
   * 刷新 Token
   * @returns 新的 Token 对
   */
  refresh(): Promise<ApiResponse<AuthTokens>> {
    return httpClient.post('/auth/refresh')
  }
}
```

### 用户 API

```typescript
// src/api/user.ts
import httpClient from '@/services/http'
import type { ApiResponse, PaginatedResponse, PaginationParams } from '@/types/api'
import type { User } from '@/types/user'

export const userApi = {
  /**
   * 获取当前用户信息
   * @returns 当前用户信息
   */
  getMe(): Promise<ApiResponse<User>> {
    return httpClient.get('/users/me')
  },

  /**
   * 更新当前用户信息
   * @param data 更新的用户信息
   * @returns 更新后的用户信息
   */
  updateMe(data: Partial<User>): Promise<ApiResponse<User>> {
    return httpClient.put('/users/me', data)
  },

  /**
   * 修改密码
   * @param data 密码信息
   */
  changePassword(data: { old_password: string; new_password: string }): Promise<ApiResponse<void>> {
    return httpClient.put('/users/me/password', data)
  },

  /**
   * 获取用户列表
   * @param params 分页参数
   * @returns 用户列表
   */
  getUsers(params?: PaginationParams): Promise<ApiResponse<PaginatedResponse<User>>> {
    return httpClient.get('/users', { params })
  },

  /**
   * 获取指定用户信息
   * @param userId 用户 ID
   * @returns 用户信息
   */
  getUser(userId: string): Promise<ApiResponse<User>> {
    return httpClient.get(`/users/${userId}`)
  },

  /**
   * 登出
   */
  logout(): Promise<ApiResponse<void>> {
    return httpClient.post('/users/logout')
  }
}
```

### 聊天室 API

```typescript
// src/api/room.ts
import httpClient from '@/services/http'
import type { ApiResponse, PaginatedResponse, PaginationParams } from '@/types/api'
import type { Room, RoomMember, CreateRoomData } from '@/types/room'
import type { Message } from '@/types/message'

export interface GetRoomsParams extends PaginationParams {
  search?: string
}

export const roomApi = {
  /**
   * 获取聊天室列表
   * @param params 查询参数
   * @returns 聊天室列表
   */
  getRooms(params?: GetRoomsParams): Promise<ApiResponse<Room[]>> {
    return httpClient.get('/rooms', { params })
  },

  /**
   * 获取最近更新的聊天室
   * @returns 聊天室列表
   */
  getRecentRooms(): Promise<ApiResponse<Room[]>> {
    return httpClient.get('/rooms/recent')
  },

  /**
   * 获取聊天室详情
   * @param roomId 聊天室 ID
   * @returns 聊天室详情
   */
  getRoom(roomId: string): Promise<ApiResponse<Room>> {
    return httpClient.get(`/rooms/${roomId}`)
  },

  /**
   * 创建聊天室
   * @param data 聊天室信息
   * @returns 新创建的聊天室
   */
  createRoom(data: CreateRoomData): Promise<ApiResponse<Room>> {
    return httpClient.post('/rooms', data)
  },

  /**
   * 更新聊天室信息
   * @param roomId 聊天室 ID
   * @param data 更新的信息
   * @returns 更新后的聊天室
   */
  updateRoom(roomId: string, data: Partial<CreateRoomData>): Promise<ApiResponse<Room>> {
    return httpClient.put(`/rooms/${roomId}`, data)
  },

  /**
   * 删除聊天室
   * @param roomId 聊天室 ID
   */
  deleteRoom(roomId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/rooms/${roomId}`)
  },

  /**
   * 加入聊天室
   * @param roomId 聊天室 ID
   */
  joinRoom(roomId: string): Promise<ApiResponse<void>> {
    return httpClient.post(`/rooms/${roomId}/join`)
  },

  /**
   * 离开聊天室
   * @param roomId 聊天室 ID
   */
  leaveRoom(roomId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/rooms/${roomId}/leave`)
  },

  /**
   * 获取聊天室成员列表
   * @param roomId 聊天室 ID
   * @returns 成员列表
   */
  getMembers(roomId: string): Promise<ApiResponse<RoomMember[]>> {
    return httpClient.get(`/rooms/${roomId}/members`)
  },

  /**
   * 踢出成员
   * @param roomId 聊天室 ID
   * @param userId 用户 ID
   */
  kickMember(roomId: string, userId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/rooms/${roomId}/members/${userId}`)
  },

  /**
   * 设置成员角色
   * @param roomId 聊天室 ID
   * @param userId 用户 ID
   * @param role 角色
   */
  setMemberRole(
    roomId: string,
    userId: string,
    role: 'admin' | 'member'
  ): Promise<ApiResponse<void>> {
    return httpClient.put(`/rooms/${roomId}/members/${userId}/role`, { role })
  },

  /**
   * 获取房间消息历史
   * @param roomId 聊天室 ID
   * @param params 分页参数
   * @returns 消息列表
   */
  getMessages(
    roomId: string,
    params?: PaginationParams
  ): Promise<ApiResponse<{ messages: Message[]; total: number; has_more: boolean }>> {
    return httpClient.get(`/rooms/${roomId}/messages`, { params })
  }
}
```

### 消息 API

```typescript
// src/api/message.ts
import httpClient from '@/services/http'
import type { ApiResponse, PaginatedResponse, PaginationParams } from '@/types/api'
import type { Message } from '@/types/message'

export interface SearchMessagesParams extends PaginationParams {
  query: string
  room_id?: string
}

export const messageApi = {
  /**
   * 搜索消息
   * @param params 搜索参数
   * @returns 消息列表
   */
  searchMessages(
    params: SearchMessagesParams
  ): Promise<ApiResponse<PaginatedResponse<Message>>> {
    return httpClient.get('/messages/search', { params })
  },

  /**
   * 编辑消息
   * @param messageId 消息 ID
   * @param content 新内容
   * @returns 更新后的消息
   */
  editMessage(messageId: string, content: string): Promise<ApiResponse<Message>> {
    return httpClient.put(`/messages/${messageId}`, { content })
  },

  /**
   * 删除消息
   * @param messageId 消息 ID
   */
  deleteMessage(messageId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/messages/${messageId}`)
  },

  /**
   * 获取消息编辑历史
   * @param messageId 消息 ID
   * @returns 编辑历史
   */
  getEditHistory(messageId: string): Promise<ApiResponse<Message[]>> {
    return httpClient.get(`/messages/${messageId}/history`)
  }
}
```

## WebSocket 服务封装

```typescript
// src/services/websocket.ts
import { ref, type Ref } from 'vue'
import { WebSocketMessageType, type WebSocketMessage } from '@/types/websocket'

type MessageHandler = (payload: unknown) => void

class WebSocketService {
  private ws: WebSocket | null = null
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectTimeout: number | null = null
  private heartbeatInterval: number | null = null
  private messageHandlers: Map<string, Set<MessageHandler>> = new Map()
  private messageQueue: WebSocketMessage[] = []
  
  public state: Ref<'CONNECTING' | 'CONNECTED' | 'DISCONNECTED'> = ref('DISCONNECTED')

  connect(token: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) return

    this.state.value = 'CONNECTING'
    const wsUrl = `${import.meta.env.VITE_WS_URL}?token=${token}`
    
    this.ws = new WebSocket(wsUrl)

    this.ws.onopen = () => {
      this.state.value = 'CONNECTED'
      this.reconnectAttempts = 0
      this.startHeartbeat()
      this.flushMessageQueue()
    }

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data)
        this.handleMessage(message)
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    this.ws.onclose = () => {
      this.state.value = 'DISCONNECTED'
      this.stopHeartbeat()
      this.attemptReconnect(token)
    }

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }
  }

  disconnect(): void {
    this.stopHeartbeat()
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout)
    }
    this.ws?.close()
    this.ws = null
    this.state.value = 'DISCONNECTED'
  }

  send(message: WebSocketMessage): boolean {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message))
      return true
    } else {
      this.messageQueue.push(message)
      return false
    }
  }

  on(type: string, handler: MessageHandler): void {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, new Set())
    }
    this.messageHandlers.get(type)!.add(handler)
  }

  off(type: string, handler: MessageHandler): void {
    this.messageHandlers.get(type)?.delete(handler)
  }

  private handleMessage(message: WebSocketMessage): void {
    const handlers = this.messageHandlers.get(message.type)
    handlers?.forEach(handler => handler(message.payload))
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = window.setInterval(() => {
      this.send({ type: 'Ping', payload: {} })
    }, 30000)
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval)
      this.heartbeatInterval = null
    }
  }

  private attemptReconnect(token: string): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached')
      return
    }

    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000)
    this.reconnectAttempts++

    this.reconnectTimeout = window.setTimeout(() => {
      this.connect(token)
    }, delay)
  }

  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift()
      if (message) {
        this.send(message)
      }
    }
  }
}

export const wsService = new WebSocketService()
```

## 错误处理规范

### 错误码映射

```typescript
// src/constants/errorCodes.ts
export const ErrorCodes = {
  // 认证错误
  AUTH_ERROR: '认证失败',
  TOKEN_EXPIRED: 'Token 已过期',
  TOKEN_INVALID: 'Token 无效',
  
  // 验证错误
  VALIDATION_ERROR: '请求参数错误',
  
  // 资源错误
  NOT_FOUND: '资源不存在',
  CONFLICT: '资源已存在',
  
  // 权限错误
  FORBIDDEN: '权限不足',
  
  // 服务器错误
  INTERNAL_ERROR: '服务器内部错误'
} as const

export function getErrorMessage(code: string): string {
  return ErrorCodes[code as keyof typeof ErrorCodes] || '未知错误'
}
```

### 统一错误处理

```typescript
// src/composables/useErrorHandler.ts
import { useMessage } from 'naive-ui'
import { getErrorMessage } from '@/constants/errorCodes'

export function useErrorHandler() {
  const message = useMessage()

  function handleError(error: unknown): void {
    if (error && typeof error === 'object' && 'response' in error) {
      const axiosError = error as { response?: { data?: { code?: string; message?: string } } }
      const code = axiosError.response?.data?.code
      const msg = axiosError.response?.data?.message
      
      if (code) {
        message.error(msg || getErrorMessage(code))
      } else {
        message.error('请求失败，请稍后重试')
      }
    } else {
      message.error('网络错误，请检查网络连接')
    }
  }

  return { handleError }
}
```
