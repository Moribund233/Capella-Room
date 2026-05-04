/**
 * WebSocket 统一配置管理模块
 */

import type { ClientConfig } from '@/types/api'

/** WebSocket 连接配置 */
export interface WebSocketConnectionConfig {
  maxReconnectAttempts: number
  reconnectInterval: number
  heartbeatInterval: number
  connectTimeout: number
}

/** 重连策略配置 */
export interface ReconnectStrategyConfig {
  baseDelayMs: number
  maxDelayMs: number
  maxAttempts: number
  multiplier: number
}

/** WebSocket 运行时状态 */
interface WebSocketRuntimeState {
  initialized: boolean
  initError: Error | null
}

// 默认配置
export const DEFAULT_CONNECTION_CONFIG: WebSocketConnectionConfig = {
  maxReconnectAttempts: 5,
  reconnectInterval: 3000,
  heartbeatInterval: 30000,
  connectTimeout: 10000,
} as const

export const DEFAULT_RECONNECT_STRATEGY: ReconnectStrategyConfig = {
  baseDelayMs: 1000,
  maxDelayMs: 30000,
  maxAttempts: 10,
  multiplier: 2,
} as const

// 内部状态
let serverConfig: ClientConfig | null = null
const runtimeState: WebSocketRuntimeState = {
  initialized: false,
  initError: null,
}
const configListeners: Array<(config: ClientConfig | null) => void> = []

/**
 * 初始化 WebSocket 配置
 */
export async function initWebSocketConfig(): Promise<boolean> {
  if (runtimeState.initialized && serverConfig) {
    return true
  }

  try {
    const { getClientConfig } = await import('@/api/system')
    serverConfig = await getClientConfig()
    runtimeState.initialized = true
    runtimeState.initError = null

    configListeners.forEach((listener) => {
      try {
        listener(serverConfig)
      } catch (error) {
        console.error('[WebSocketConfig] 配置监听器执行失败:', error)
      }
    })

    return true
  } catch (error) {
    runtimeState.initError = error instanceof Error ? error : new Error('初始化失败')
    console.warn('[WebSocketConfig] 使用默认配置')
    return false
  }
}

/**
 * 检查是否已初始化
 */
export function isConfigInitialized(): boolean {
  return runtimeState.initialized
}

/**
 * 获取连接配置
 */
export function getConnectionConfig(): WebSocketConnectionConfig {
  if (serverConfig?.websocket) {
    return {
      maxReconnectAttempts: serverConfig.reconnect?.max_attempts ?? DEFAULT_CONNECTION_CONFIG.maxReconnectAttempts,
      reconnectInterval: serverConfig.reconnect?.base_delay_ms ?? DEFAULT_CONNECTION_CONFIG.reconnectInterval,
      heartbeatInterval: (serverConfig.websocket.heartbeat_interval_secs ?? 30) * 1000,
      connectTimeout: DEFAULT_CONNECTION_CONFIG.connectTimeout,
    }
  }
  return DEFAULT_CONNECTION_CONFIG
}

/**
 * 获取心跳超时时间（毫秒）
 * 使用服务端配置的 heartbeat_timeout_secs，并留出 80% 的缓冲时间
 */
export function getHeartbeatTimeoutMs(): number {
  const timeoutSecs = serverConfig?.websocket?.heartbeat_timeout_secs ?? 90
  // 使用服务端超时时间的 80% 作为客户端检测阈值（留出缓冲）
  return Math.floor(timeoutSecs * 1000 * 0.8)
}

/**
 * 获取重连策略
 */
export function getReconnectStrategy(): ReconnectStrategyConfig {
  return DEFAULT_RECONNECT_STRATEGY
}

/**
 * 计算重连延迟
 */
export function calculateReconnectDelay(attempt: number): number {
  const strategy = getReconnectStrategy()
  const delay = strategy.baseDelayMs * Math.pow(strategy.multiplier, attempt)
  return Math.min(delay, strategy.maxDelayMs)
}

/**
 * 添加配置变更监听器
 */
export function addConfigListener(listener: (config: ClientConfig | null) => void): void {
  configListeners.push(listener)
}

/**
 * 移除配置变更监听器
 */
export function removeConfigListener(listener: (config: ClientConfig | null) => void): void {
  const index = configListeners.indexOf(listener)
  if (index > -1) {
    configListeners.splice(index, 1)
  }
}
