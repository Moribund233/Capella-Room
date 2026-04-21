/**
 * WebSocket Store
 * 集中管理 WebSocket 业务逻辑和状态
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { getAccessToken, getWebSocketClient } from '@/api'
import { useAuthStore } from './auth'
import type {
  WebSocketMessage,
  ChatMessageData,
  WebSocketUserInfo,
  UserStatus,
} from '@/types/websocket'

export type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting'

export interface ChatMessage {
  id: string
  type: 'sent' | 'received' | 'system'
  content: string
  time: string
  sender?: WebSocketUserInfo
  roomId?: string
}

export const useWebSocketStore = defineStore('websocket', () => {
  // ========== State ==========
  const status = ref<WebSocketStatus>('disconnected')
  const lastError = ref<string | null>(null)
  const reconnectAttempts = ref(0)

  // 房间相关状态
  const joinedRooms = ref<string[]>([])
  const currentRoom = ref<string | null>(null)
  const chatMessages = ref<ChatMessage[]>([])
  const onlineUsers = ref<WebSocketUserInfo[]>([])
  const currentUserStatus = ref<UserStatus>('offline')

  // WebSocket 客户端实例
  const wsClient = getWebSocketClient()

  // ========== Getters ==========
  const isConnected = computed(() => status.value === 'connected')
  const isConnecting = computed(() => status.value === 'connecting' || status.value === 'reconnecting')

  // ========== Actions ==========

  /**
   * 连接到 WebSocket
   */
  function connect() {
    if (wsClient.isConnected() || status.value === 'connecting') {
      return
    }

    const token = getAccessToken()
    if (!token) {
      console.warn('未登录，无法连接 WebSocket')
      lastError.value = '未登录'
      return
    }

    status.value = reconnectAttempts.value > 0 ? 'reconnecting' : 'connecting'

    wsClient.setHandlers({
      onConnect: () => {
        console.log('[WebSocket] 连接成功')
        status.value = 'connected'
        reconnectAttempts.value = 0
        lastError.value = null
      },
      onDisconnect: () => {
        console.log('[WebSocket] 连接断开')
        status.value = 'disconnected'
        joinedRooms.value = []
        currentRoom.value = null
        onlineUsers.value = []
      },
      onError: (error: Error) => {
        console.error('[WebSocket] 错误:', error)
        lastError.value = '连接错误'
      },
      onMessage: (message: WebSocketMessage) => {
        handleMessage(message)
      }
    })

    wsClient.connect().catch((error: Error) => {
      console.error('[WebSocket] 连接失败:', error)
      status.value = 'disconnected'
    })
  }

  /**
   * 断开 WebSocket 连接
   */
  function disconnect() {
    reconnectAttempts.value = 0
    wsClient.disconnect()
    status.value = 'disconnected'
    joinedRooms.value = []
    currentRoom.value = null
    onlineUsers.value = []
  }

  /**
   * 加入房间
   */
  function joinRoom(roomId: string) {
    if (!isConnected.value) {
      console.warn('WebSocket 未连接，无法加入房间')
      return
    }

    wsClient.send({
      type: 'JoinRoom',
      payload: { room_id: roomId }
    })

    if (!joinedRooms.value.includes(roomId)) {
      joinedRooms.value.push(roomId)
    }
    currentRoom.value = roomId
  }

  /**
   * 离开房间
   */
  function leaveRoom(roomId: string) {
    if (!isConnected.value) {
      return
    }

    wsClient.send({
      type: 'LeaveRoom',
      payload: { room_id: roomId }
    })

    joinedRooms.value = joinedRooms.value.filter(id => id !== roomId)
    if (currentRoom.value === roomId) {
      currentRoom.value = null
    }
  }

  /**
   * 发送消息
   */
  function sendMessage(roomId: string, content: string) {
    if (!isConnected.value) {
      console.warn('WebSocket 未连接，无法发送消息')
      return
    }

    wsClient.send({
      type: 'ChatMessage',
      payload: {
        room_id: roomId,
        content
      }
    })

    // 本地添加消息
    const authStore = useAuthStore()
    chatMessages.value.push({
      id: Date.now().toString(),
      type: 'sent',
      content,
      time: new Date().toISOString(),
      sender: authStore.user ? {
        id: authStore.user.id,
        username: authStore.user.username,
        avatar_url: authStore.user.avatar_url || undefined,
        status: 'online'
      } : undefined,
      roomId
    })
  }

  /**
   * 设置用户状态
   */
  function setUserStatus(newStatus: UserStatus) {
    if (!isConnected.value) {
      console.warn('WebSocket 未连接，无法设置状态')
      return
    }

    wsClient.send({
      type: 'StatusUpdate',
      payload: { status: newStatus }
    })

    currentUserStatus.value = newStatus
  }

  /**
   * 处理收到的消息
   */
  function handleMessage(message: WebSocketMessage) {
    switch (message.type) {
      case 'Chat':
      case 'ChatMessage': {
        const data = message.payload as ChatMessageData
        chatMessages.value.push({
          id: data.id,
          type: 'received',
          content: data.content,
          time: data.created_at,
          sender: data.sender,
          roomId: data.room_id
        })
        break
      }

      case 'System': {
        const data = message.payload as { message: string }
        chatMessages.value.push({
          id: Date.now().toString(),
          type: 'system',
          content: data.message,
          time: new Date().toISOString()
        })
        break
      }

      case 'RoomJoined': {
        const data = message.payload as { room_id: string; members: WebSocketUserInfo[] }
        if (!joinedRooms.value.includes(data.room_id)) {
          joinedRooms.value.push(data.room_id)
        }
        onlineUsers.value = data.members
        break
      }

      case 'RoomLeft': {
        const data = message.payload as { room_id: string }
        joinedRooms.value = joinedRooms.value.filter(id => id !== data.room_id)
        if (currentRoom.value === data.room_id) {
          currentRoom.value = null
        }
        break
      }

      case 'UserJoined': {
        const data = message.payload as { user: WebSocketUserInfo }
        if (!onlineUsers.value.find(u => u.id === data.user.id)) {
          onlineUsers.value.push(data.user)
        }
        break
      }

      case 'UserLeft': {
        const data = message.payload as { user_id: string }
        onlineUsers.value = onlineUsers.value.filter(u => u.id !== data.user_id)
        break
      }

      case 'UserStatusUpdate': {
        const data = message.payload as { user_id: string; status: UserStatus }
        const user = onlineUsers.value.find(u => u.id === data.user_id)
        if (user) {
          user.status = data.status
        }
        break
      }

      case 'Error': {
        const data = message.payload as { message: string }
        lastError.value = data.message
        console.error('[WebSocket] 错误:', data.message)
        break
      }
    }
  }

  return {
    // State
    status,
    lastError,
    reconnectAttempts,
    joinedRooms,
    currentRoom,
    chatMessages,
    onlineUsers,
    currentUserStatus,
    // Getters
    isConnected,
    isConnecting,
    // Actions
    connect,
    disconnect,
    joinRoom,
    leaveRoom,
    sendMessage,
    setUserStatus
  }
})
