/**
 * WebSocket Store
 * 集中管理 WebSocket 业务逻辑和状态
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { getAccessToken, getWebSocketClient, resetWebSocketClient } from '@/api'
import { useAuthStore } from './auth'
import type {
  WebSocketMessage,
  ChatMessageData,
  WebSocketUserInfo,
  UserStatus,
} from '@/types/websocket'

export type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting'

export interface ChatMessage {
  id?: string
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
      onAuthFailed: (error: Error) => {
        console.error('[WebSocket] 认证失败，需要重新登录:', error)
        status.value = 'disconnected'
        lastError.value = '登录已过期，请重新登录'
        // Token 过期，完全重置 WebSocket 客户端，阻止自动重连
        resetWebSocketClient()
        // 清除认证状态并跳转到登录页
        const authStore = useAuthStore()
        authStore.logout()
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
    if (!isConnected.value || !roomId) {
      console.warn('WebSocket 未连接或房间ID无效，无法加入房间')
      return
    }

    // 后端期望邻接标签格式: { type: "JoinRoom", payload: { room_id: "..." } }
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
    if (!isConnected.value || !roomId) {
      return
    }

    // 后端期望邻接标签格式: { type: "LeaveRoom", payload: { room_id: "..." } }
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
   * 注意：消息不会立即显示在本地，而是等待服务器广播 NewMessage
   */
  function sendMessage(roomId: string, content: string) {
    if (!isConnected.value || !roomId) {
      console.warn('WebSocket 未连接或房间ID无效，无法发送消息')
      return
    }

    // 后端期望邻接标签格式: { type: "ChatMessage", payload: { room_id: "...", content: "..." } }
    wsClient.send({
      type: 'ChatMessage',
      payload: {
        room_id: roomId,
        content
      }
    })
    // 不本地添加消息，等待服务器广播 NewMessage
  }

  /**
   * 设置用户状态
   */
  function setUserStatus(newStatus: UserStatus) {
    if (!isConnected.value || !newStatus) {
      console.warn('WebSocket 未连接或状态无效，无法设置状态')
      return
    }

    // 后端期望邻接标签格式: { type: "UpdateStatus", payload: { status: "..." } }
    wsClient.send({
      type: 'UpdateStatus',
      payload: { status: newStatus }
    })

    currentUserStatus.value = newStatus
  }

  /**
   * 加载历史消息
   */
  function loadHistoryMessages(roomId: string, messages: ChatMessage[]) {
    // 过滤掉已存在的消息（避免重复）
    const existingIds = new Set(chatMessages.value.map(m => m.id))
    const newMessages = messages.filter(m => !existingIds.has(m.id))
    // 按时间排序后添加到消息列表
    chatMessages.value.push(...newMessages)
    // 按时间排序
    chatMessages.value.sort((a, b) => new Date(a.time).getTime() - new Date(b.time).getTime())
  }

  /**
   * 处理收到的消息
   */
  function handleMessage(message: WebSocketMessage) {
    switch (message.type) {
      case 'NewMessage': {
        const data = message.payload as ChatMessageData
        const authStore = useAuthStore()

        // 检查是否是自己发送的消息
        const isOwnMessage = data.sender_id === authStore.user?.id ||
                            data.sender?.id === authStore.user?.id

        // 如果消息已存在（通过ID检查），则跳过
        const messageId = data.message_id || data.id
        const exists = chatMessages.value.some(m => m.id === messageId)
        if (exists) {
          break
        }

        // 构建 sender 信息（优先使用 sender 对象，否则用 sender_id/sender_name）
        const sender = data.sender || {
          id: data.sender_id || '',
          username: data.sender_name || '未知用户',
          status: 'online' as UserStatus
        }

        chatMessages.value.push({
          id: messageId || Date.now().toString(),
          type: isOwnMessage ? 'sent' : 'received',
          content: data.content,
          time: data.created_at || new Date().toISOString(),
          sender,
          roomId: data.room_id
        })
        break
      }

      case 'SystemMessage': {
        const data = message.payload as { content: string }
        chatMessages.value.push({
          id: Date.now().toString(),
          type: 'system',
          content: data.content,
          time: new Date().toISOString()
        })
        break
      }

      case 'RoomJoined': {
        const data = message.payload as { room_id: string; user_id: string; username: string }
        if (!joinedRooms.value.includes(data.room_id)) {
          joinedRooms.value.push(data.room_id)
        }
        currentRoom.value = data.room_id
        // RoomJoined only contains current user info, online users come from OnlineUsers message
        break
      }

      case 'OnlineUsers': {
        const data = message.payload as { room_id: string; users: WebSocketUserInfo[] }
        onlineUsers.value = data.users
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
        const data = message.payload as { room_id: string; user_id: string; username: string }
        if (!onlineUsers.value.find(u => u.id === data.user_id)) {
          onlineUsers.value.push({
            id: data.user_id,
            username: data.username,
            status: 'online'
          })
        }
        break
      }

      case 'UserLeft': {
        const data = message.payload as { user_id: string }
        onlineUsers.value = onlineUsers.value.filter(u => u.id !== data.user_id)
        break
      }

      case 'UserStatusChanged': {
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
    setUserStatus,
    loadHistoryMessages
  }
})
