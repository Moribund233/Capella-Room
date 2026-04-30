import { ref, computed } from 'vue'
import type { WsMessage, WsAuthMessage, WsAuthResult, WsConnectionState } from '@/types'
import { getClientConfig, getHeartbeatTimeoutMs, type ClientConfig } from './config'

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
  private heartbeatTimeout: number | null = null
  private reconnectTimer: number | null = null
  private messageHandlers: Map<string, ((payload: unknown) => void)[]> = new Map()
  private clientConfig: ClientConfig | null = null
  private lastPongTime = 0

  // 响应式状态
  public connectionState = ref<WsConnectionState>('disconnected')
  public isConnected = computed(() => this.connectionState.value === 'connected' || this.connectionState.value === 'authenticated')
  public isAuthenticated = computed(() => this.connectionState.value === 'authenticated')

  /**
   * 获取 WebSocket 配置
   */
  private get wsConfig() {
    return this.clientConfig?.websocket
  }

  /**
   * 获取重连配置
   */
  private get reconnectConfig() {
    return this.clientConfig?.reconnect
  }

  /**
   * 加载客户端配置
   */
  async loadConfig(): Promise<void> {
    try {
      const res = await getClientConfig()
      if (res.success && res.data) {
        this.clientConfig = res.data
        console.log('[WebSocket] 配置已加载:', this.clientConfig)
      }
    } catch (error) {
      console.error('[WebSocket] 加载配置失败，使用默认值:', error)
    }
  }

  /**
   * 建立 WebSocket 连接
   * @param token JWT 访问令牌
   */
  async connect(token: string): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.log('[WebSocket] 连接已存在')
      return
    }

    // 确保配置已加载
    if (!this.clientConfig) {
      await this.loadConfig()
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
        this.stopHeartbeatTimeout()
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
          // 认证成功后启动心跳超时检测
          this.startHeartbeatTimeout()
        } else {
          console.error('[WebSocket] 认证失败:', result.error)
          this.connectionState.value = 'disconnected'
        }
        return
      }

      // 处理服务端发送的 Ping - 回复 Pong
      if (message.type === 'Ping') {
        this.send({ type: 'Pong' })
        // 更新最后收到心跳的时间（收到 Ping 表示连接正常）
        this.lastPongTime = Date.now()
        console.log('[WebSocket] 收到 Ping，已回复 Pong')
        return
      }

      // 处理 Pong（如果服务端也发送 Pong）
      if (message.type === 'Pong') {
        this.lastPongTime = Date.now()
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
   * 启动心跳超时检测
   * 后端机制：服务端每30秒发送Ping，90秒未收到Pong则断开
   * 前端：检测是否收到服务端Ping，如果超过阈值没有收到则认为超时
   */
  private startHeartbeatTimeout(): void {
    this.stopHeartbeatTimeout()

    // 使用服务端配置的超时时间（带缓冲），默认90秒的80%即72秒
    const timeoutMs = getHeartbeatTimeoutMs(this.clientConfig)

    this.lastPongTime = Date.now()

    // 定期检查是否超时
    const checkInterval = 10000 // 每10秒检查一次
    this.heartbeatTimeout = window.setInterval(() => {
      const elapsed = Date.now() - this.lastPongTime
      if (elapsed > timeoutMs) {
        console.error(`[WebSocket] 心跳超时，${elapsed}ms 未收到服务端消息`)
        this.disconnect()
        // 触发重连
        // 注意：这里需要通过其他方式获取token进行重连
      }
    }, checkInterval)

    console.log(`[WebSocket] 心跳超时检测已启动，超时时间: ${timeoutMs}ms`)
  }

  /**
   * 停止心跳超时检测
   */
  private stopHeartbeatTimeout(): void {
    if (this.heartbeatTimeout) {
      clearInterval(this.heartbeatTimeout)
      this.heartbeatTimeout = null
    }
  }

  /**
   * 尝试重新连接
   * @param token JWT 访问令牌
   */
  private attemptReconnect(token: string): void {
    // 使用配置的最大重连次数，默认5次
    const maxAttempts = this.reconnectConfig?.max_attempts || 5

    if (this.reconnectAttempts >= maxAttempts) {
      console.error('[WebSocket] 重连次数已达上限')
      return
    }

    this.reconnectAttempts++
    this.connectionState.value = 'reconnecting'

    // 使用配置的重连延迟，默认3000ms
    const baseDelay = this.reconnectConfig?.base_delay_ms || 3000
    const delay = baseDelay * this.reconnectAttempts

    console.log(`[WebSocket] ${delay}ms 后尝试第 ${this.reconnectAttempts} 次重连...`)

    this.reconnectTimer = window.setTimeout(() => {
      this.connect(token)
    }, delay)
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    this.stopHeartbeatTimeout()

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
    loadConfig: () => wsClient.loadConfig(),
  }
}
