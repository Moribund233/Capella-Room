/**
 * 多用户 WebSocket 管理器（优化版）
 * 复用 WebSocketClient，专注状态管理和业务逻辑
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { WebSocketClient } from '@/api/websocket'
import { useMultiUserAuthStore } from './multiUserAuth'
import {
  getConnectionConfig,
  initWebSocketConfig,
  isConfigInitialized,
} from '@/config/websocketConfig'
import type { WebSocketMessage } from '@/types/websocket'

export type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'reconnecting'

/**
 * 单个用户的 WebSocket 状态
 */
export interface UserWebSocketState {
  userId: string
  username: string
  status: WebSocketStatus
  client: WebSocketClient | null
  lastError: string | null
  connectedAt: number | null
  joinedRooms: string[]
}

/**
 * 消息处理器
 */
export type MessageHandler = (userId: string, message: WebSocketMessage) => void

export const useMultiUserWebSocketStore = defineStore('multiUserWebSocket', () => {
  // ========== State ==========
  const userConnections = ref<Map<string, UserWebSocketState>>(new Map())
  const messageHandlers = ref<MessageHandler[]>([])

  // ========== Getters ==========
  const connections = computed(() => Array.from(userConnections.value.values()))
  const connectedUsers = computed(() => connections.value.filter(c => c.status === 'connected'))
  const connectionCount = computed(() => userConnections.value.size)
  const connectedCount = computed(() => connectedUsers.value.length)

  /**
   * 是否所有用户都已连接
   */
  const allConnected = computed(() => {
    const authStore = useMultiUserAuthStore()
    return authStore.testUsers.length > 0 &&
           authStore.testUsers.every(u => userConnections.value.get(u.id)?.status === 'connected')
  })

  // ========== 核心方法 ==========

  /**
   * 为用户创建 WebSocket 连接（由 multiUserAuth 在认证成功后调用）
   * @param userId 用户ID
   * @param accessToken 访问令牌
   */
  async function createConnection(userId: string, accessToken: string): Promise<void> {
    const authStore = useMultiUserAuthStore()
    const user = authStore.getUser(userId)
    if (!user) throw new Error(`用户 ${userId} 未找到`)

    // 确保配置已初始化
    if (!isConfigInitialized()) {
      await initWebSocketConfig()
    }

    // 如果已存在连接，先断开
    const existingState = userConnections.value.get(userId)
    if (existingState?.client) {
      existingState.client.disconnect()
    }

    // 创建新的状态对象
    const state: UserWebSocketState = {
      userId,
      username: user.username,
      status: 'connecting',
      client: null,
      lastError: null,
      connectedAt: null,
      joinedRooms: [],
    }
    userConnections.value.set(userId, state)

    // 从统一配置模块获取连接配置
    const config = getConnectionConfig()

    // 创建 WebSocketClient 实例（复用标准连接层）
    const client = new WebSocketClient(
      {
        maxReconnectAttempts: config.maxReconnectAttempts,
        reconnectInterval: config.reconnectInterval,
        heartbeatInterval: config.heartbeatInterval,
        connectTimeout: config.connectTimeout,
      },
      {
        onConnect: () => {
          // WebSocketClient 的 connect() 现在会在认证成功后才 resolve
          // onConnect 回调在认证成功后触发，状态已经设置为 connected
          state.status = 'connected'
          state.connectedAt = Date.now()
          console.log(`[MultiWS] 用户 ${user.username} 连接并认证成功`)
          // 注意：认证消息已由 WebSocketClient 在连接时自动发送，无需重复发送
        },
        onDisconnect: () => {
          state.status = 'disconnected'
          state.connectedAt = null
          state.joinedRooms = []  // 断开时清空已加入房间列表
          console.log(`[MultiWS] 用户 ${user.username} 连接断开`)
        },
        onError: (error) => {
          state.status = 'error'
          state.lastError = error.message
          console.error(`[MultiWS] 用户 ${user.username} 连接错误:`, error)
        },
        onMessage: (message) => {
          handleServerMessage(userId, message)
        },
      }
    )

    // 设置用户 token
    client.setToken(accessToken)
    state.client = client

    // 执行连接
    await client.connect()
  }

  /**
   * 处理服务器消息
   */
  function handleServerMessage(userId: string, message: WebSocketMessage): void {
    const state = userConnections.value.get(userId)
    if (!state) return

    // 处理特定消息类型来跟踪状态
    switch (message.type) {
      case 'AuthResult':
        if ((message.payload as { success: boolean }).success) {
          console.log(`[MultiWS] 用户 ${state.username} WS 认证成功`)
        }
        break

      case 'RoomJoined':
      case 'UserJoined': {
        const roomId = (message.payload as { room_id: string })?.room_id
        if (roomId && !state.joinedRooms.includes(roomId)) {
          state.joinedRooms.push(roomId)
          console.log(`[MultiWS] 用户 ${state.username} 加入房间 ${roomId}`)
        }
        break
      }

      case 'RoomLeft':
      case 'UserLeft': {
        const roomId = (message.payload as { room_id: string })?.room_id
        if (roomId) {
          const index = state.joinedRooms.indexOf(roomId)
          if (index > -1) {
            state.joinedRooms.splice(index, 1)
            console.log(`[MultiWS] 用户 ${state.username} 离开房间 ${roomId}`)
          }
        }
        break
      }

      case 'Error': {
        const errorMsg = (message.payload as { message?: string })?.message
        if (errorMsg) {
          console.error(`[MultiWS] 用户 ${state.username} 收到错误:`, errorMsg)
        }
        break
      }
    }

    // 分发到所有注册的消息处理器
    for (const handler of messageHandlers.value) {
      try {
        handler(userId, message)
      } catch (error) {
        console.error('[MultiWS] 消息处理器错误:', error)
      }
    }
  }

  /**
   * 以指定用户身份发送消息
   */
  function sendAsUser(userId: string, message: WebSocketMessage): boolean {
    const state = userConnections.value.get(userId)
    return state?.client?.send(message) ?? false
  }

  /**
   * 发送聊天消息
   */
  function sendChatMessage(userId: string, roomId: string, content: string): boolean {
    return sendAsUser(userId, {
      type: 'ChatMessage',
      payload: { room_id: roomId, content },
    })
  }

  /**
   * 加入房间
   */
  function joinRoom(userId: string, roomId: string): boolean {
    return sendAsUser(userId, {
      type: 'JoinRoom',
      payload: { room_id: roomId },
    })
  }

  /**
   * 离开房间
   */
  function leaveRoom(userId: string, roomId: string): boolean {
    return sendAsUser(userId, {
      type: 'LeaveRoom',
      payload: { room_id: roomId },
    })
  }

  /**
   * 断开用户连接
   */
  function disconnectUser(userId: string): void {
    const state = userConnections.value.get(userId)
    if (state?.client) {
      state.client.disconnect()
      state.client = null
      state.status = 'disconnected'
    }
  }

  /**
   * 断开所有连接
   */
  function disconnectAll(): void {
    for (const state of userConnections.value.values()) {
      state.client?.disconnect()
    }
    userConnections.value.clear()
  }

  // ========== 查询方法 ==========

  /**
   * 获取用户连接状态
   */
  function getUserStatus(userId: string): WebSocketStatus {
    return userConnections.value.get(userId)?.status || 'disconnected'
  }

  /**
   * 检查用户是否已连接
   */
  function isUserConnected(userId: string): boolean {
    return getUserStatus(userId) === 'connected'
  }

  /**
   * 检查用户是否已加入指定房间
   */
  function isUserInRoom(userId: string, roomId: string): boolean {
    return userConnections.value.get(userId)?.joinedRooms.includes(roomId) ?? false
  }

  /**
   * 获取用户已加入的房间列表
   */
  function getUserJoinedRooms(userId: string): string[] {
    return userConnections.value.get(userId)?.joinedRooms ?? []
  }

  /**
   * 注册消息处理器
   */
  function onMessage(handler: MessageHandler): () => void {
    messageHandlers.value.push(handler)
    return () => {
      const index = messageHandlers.value.indexOf(handler)
      if (index > -1) {
        messageHandlers.value.splice(index, 1)
      }
    }
  }

  return {
    // State
    userConnections,
    // Getters
    connections,
    connectedUsers,
    connectionCount,
    connectedCount,
    allConnected,
    // Actions
    createConnection,
    disconnectUser,
    disconnectAll,
    sendAsUser,
    sendChatMessage,
    joinRoom,
    leaveRoom,
    onMessage,
    getUserStatus,
    isUserConnected,
    isUserInRoom,
    getUserJoinedRooms,
  }
})
