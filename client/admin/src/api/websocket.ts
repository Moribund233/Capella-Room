import { ref, computed } from 'vue'
import type { WsMessage, WsAuthMessage, WsAuthResult, WsConnectionState } from '@/types'

/**
 * WebSocket 基础 URL
 */
const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080/ws'

/**
 * WebSocket 客户端类
 */
class WebSocketClient {
  private ws: WebSocket | null = null
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 3000
  private heartbeatInterval: number | null = null
  private reconnectTimer: number | null = null
  private messageHandlers: Map<string, ((payload: unknown) => void)[]> = new Map()

  // 响应式状态
  public connectionState = ref<WsConnectionState>('disconnected')
  public isConnected = computed(() => this.connectionState.value === 'connected' || this.connectionState.value === 'authenticated')
  public isAuthenticated = computed(() => this.connectionState.value === 'authenticated')

  /**
   * 建立 WebSocket 连接
   * @param token JWT 访问令牌
   */
  connect(token: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.log('[WebSocket] 连接已存在')
      return
    }

    this.connectionState.value = 'connecting'

    try {
      this.ws = new WebSocket(WS_BASE_URL)

      this.ws.onopen = () => {
        console.log('[WebSocket] 连接已建立')
        this.connectionState.value = 'connected'
        this.reconnectAttempts = 0
        // 发送认证消息
        this.authenticate(token)
      }

      this.ws.onmessage = (event) => {
        this.handleMessage(event.data)
      }

      this.ws.onclose = () => {
        console.log('[WebSocket] 连接已关闭')
        this.connectionState.value = 'disconnected'
        this.stopHeartbeat()
        this.attemptReconnect(token)
      }

      this.ws.onerror = (error) => {
        console.error('[WebSocket] 连接错误:', error)
        this.connectionState.value = 'disconnected'
      }
    } catch (error) {
      console.error('[WebSocket] 连接失败:', error)
      this.connectionState.value = 'disconnected'
      this.attemptReconnect(token)
    }
  }

  /**
   * 发送认证消息
   * @param token JWT 访问令牌
   */
  private authenticate(token: string): void {
    const authMessage: WsMessage<WsAuthMessage> = {
      type: 'Auth',
      payload: { token },
    }
    this.send(authMessage)
  }

  /**
   * 处理收到的消息
   * @param data 消息数据
   */
  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data) as WsMessage<unknown>
      console.log('[WebSocket] 收到消息:', message)

      // 处理认证结果
      if (message.type === 'AuthResult') {
        const result = message.payload as WsAuthResult
        if (result.success) {
          console.log('[WebSocket] 认证成功')
          this.connectionState.value = 'authenticated'
          this.startHeartbeat()
        } else {
          console.error('[WebSocket] 认证失败:', result.error)
          this.connectionState.value = 'disconnected'
        }
        return
      }

      // 处理心跳响应
      if (message.type === 'Pong') {
        return
      }

      // 分发消息到对应的处理器
      const handlers = this.messageHandlers.get(message.type)
      if (handlers) {
        handlers.forEach((handler) => handler(message.payload))
      }
    } catch (error) {
      console.error('[WebSocket] 消息解析失败:', error)
    }
  }

  /**
   * 发送消息
   * @param message WebSocket 消息
   */
  send<T>(message: WsMessage<T>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message))
    } else {
      console.warn('[WebSocket] 连接未就绪，无法发送消息')
    }
  }

  /**
   * 订阅消息
   * @param type 消息类型
   * @param handler 消息处理器
   */
  on<T>(type: string, handler: (payload: T) => void): void {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, [])
    }
    this.messageHandlers.get(type)?.push(handler as (payload: unknown) => void)
  }

  /**
   * 取消订阅
   * @param type 消息类型
   * @param handler 消息处理器
   */
  off<T>(type: string, handler: (payload: T) => void): void {
    const handlers = this.messageHandlers.get(type)
    if (handlers) {
      const index = handlers.indexOf(handler as (payload: unknown) => void)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  /**
   * 启动心跳
   */
  private startHeartbeat(): void {
    this.heartbeatInterval = window.setInterval(() => {
      this.send({ type: 'Ping' })
    }, 30000) // 每 30 秒发送一次心跳
  }

  /**
   * 停止心跳
   */
  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval)
      this.heartbeatInterval = null
    }
  }

  /**
   * 尝试重新连接
   * @param token JWT 访问令牌
   */
  private attemptReconnect(token: string): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WebSocket] 重连次数已达上限')
      return
    }

    this.reconnectAttempts++
    this.connectionState.value = 'reconnecting'
    console.log(`[WebSocket] ${this.reconnectDelay}ms 后尝试第 ${this.reconnectAttempts} 次重连...`)

    this.reconnectTimer = window.setTimeout(() => {
      this.connect(token)
    }, this.reconnectDelay)
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    this.stopHeartbeat()

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.ws) {
      this.ws.close()
      this.ws = null
    }

    this.connectionState.value = 'disconnected'
    this.messageHandlers.clear()
    console.log('[WebSocket] 已断开连接')
  }
}

/**
 * WebSocket 客户端单例
 */
export const wsClient = new WebSocketClient()

/**
 * 使用 WebSocket 的组合式函数
 */
export function useWebSocket() {
  return {
    client: wsClient,
    connectionState: wsClient.connectionState,
    isConnected: wsClient.isConnected,
    isAuthenticated: wsClient.isAuthenticated,
    connect: (token: string) => wsClient.connect(token),
    disconnect: () => wsClient.disconnect(),
    send: <T>(message: WsMessage<T>) => wsClient.send(message),
    on: <T>(type: string, handler: (payload: T) => void) => wsClient.on(type, handler),
    off: <T>(type: string, handler: (payload: T) => void) => wsClient.off(type, handler),
  }
}
