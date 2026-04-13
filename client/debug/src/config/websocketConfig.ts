/**
 * WebSocket 统一配置管理模块
 * 集中管理前端 WebSocket 相关配置，支持从服务端动态获取配置
 * 供标准业务层和多用户业务层共享使用
 */

import { ref, computed, type Ref, type ComputedRef } from 'vue'
import { getClientConfig } from '@/api/system'
import type { ClientConfig } from '@/types/config'

/** WebSocket 连接配置 */
export interface WebSocketConnectionConfig {
  /** 最大重连次数 */
  maxReconnectAttempts: number
  /** 重连间隔（毫秒）- 固定间隔模式使用 */
  reconnectInterval: number
  /** 心跳间隔（毫秒） */
  heartbeatInterval: number
  /** 连接超时（毫秒） */
  connectTimeout: number
}

/** 重连策略配置 */
export interface ReconnectStrategyConfig {
  /** 基础延迟（毫秒） */
  baseDelayMs: number
  /** 最大延迟（毫秒） */
  maxDelayMs: number
  /** 最大重连次数 */
  maxAttempts: number
  /** 延迟倍数（指数退避） */
  multiplier: number
}

/** WebSocket 运行时状态 */
interface WebSocketRuntimeState {
  /** 是否已初始化 */
  initialized: boolean
  /** 初始化错误 */
  initError: Error | null
}

// ==================== 默认值配置 ====================

/** 默认 WebSocket 连接配置 */
export const DEFAULT_CONNECTION_CONFIG: WebSocketConnectionConfig = {
  maxReconnectAttempts: 5,
  reconnectInterval: 3000,
  heartbeatInterval: 30000,
  connectTimeout: 10000,
} as const

/** 默认重连策略配置 */
export const DEFAULT_RECONNECT_STRATEGY: ReconnectStrategyConfig = {
  baseDelayMs: 1000,
  maxDelayMs: 30000,
  maxAttempts: 10,
  multiplier: 2,
} as const

// ==================== 内部状态 ====================

/** 服务端配置缓存 */
let serverConfig: ClientConfig | null = null

/** 运行时状态 */
const runtimeState: WebSocketRuntimeState = {
  initialized: false,
  initError: null,
}

/** 配置变更监听器 */
const configListeners: Array<(config: ClientConfig | null) => void> = []

// ==================== 核心函数 ====================

/**
 * 初始化 WebSocket 配置
 * 从服务端获取配置并缓存，应在应用启动时调用
 * @returns 是否成功初始化
 */
export async function initWebSocketConfig(): Promise<boolean> {
  if (runtimeState.initialized && serverConfig) {
    console.log('[WebSocketConfig] 配置已初始化，跳过')
    return true
  }

  try {
    serverConfig = await getClientConfig()
    runtimeState.initialized = true
    runtimeState.initError = null

    console.log('[WebSocketConfig] 已加载服务端配置:', serverConfig)

    // 通知所有监听器
    configListeners.forEach((listener) => {
      try {
        listener(serverConfig)
      } catch (error) {
        console.error('[WebSocketConfig] 配置监听器执行失败:', error)
      }
    })

    return true
  } catch (error) {
    runtimeState.initialized = true
    runtimeState.initError = error instanceof Error ? error : new Error(String(error))

    console.warn('[WebSocketConfig] 无法获取服务端配置，使用默认值:', error)
    return false
  }
}

/**
 * 重新加载配置
 * 用于配置热更新场景
 */
export async function reloadWebSocketConfig(): Promise<boolean> {
  runtimeState.initialized = false
  serverConfig = null
  return initWebSocketConfig()
}

/**
 * 注册配置变更监听器
 * @param listener 监听器回调函数
 * @returns 取消订阅函数
 */
export function onConfigChange(listener: (config: ClientConfig | null) => void): () => void {
  configListeners.push(listener)

  // 如果已经初始化，立即通知
  if (runtimeState.initialized) {
    listener(serverConfig)
  }

  return () => {
    const index = configListeners.indexOf(listener)
    if (index > -1) {
      configListeners.splice(index, 1)
    }
  }
}

// ==================== 配置获取函数 ====================

/**
 * 获取原始服务端配置
 * @returns 服务端配置或 null（如果未获取成功）
 */
export function getServerConfig(): ClientConfig | null {
  return serverConfig
}

/**
 * 获取 WebSocket 连接配置（用于 WebSocketClient）
 * 合并服务端配置和默认值
 */
export function getConnectionConfig(): WebSocketConnectionConfig {
  if (!serverConfig) {
    return { ...DEFAULT_CONNECTION_CONFIG }
  }

  return {
    maxReconnectAttempts:
      serverConfig.reconnect.max_attempts ?? DEFAULT_CONNECTION_CONFIG.maxReconnectAttempts,
    reconnectInterval: DEFAULT_CONNECTION_CONFIG.reconnectInterval,
    heartbeatInterval:
      (serverConfig.websocket.heartbeat_interval_secs ?? 30) * 1000,
    connectTimeout:
      (serverConfig.websocket.auth_timeout_secs ?? 10) * 1000,
  }
}

/**
 * 获取重连策略配置
 * 用于自定义重连逻辑（如指数退避）
 */
export function getReconnectStrategy(): ReconnectStrategyConfig {
  if (!serverConfig) {
    return { ...DEFAULT_RECONNECT_STRATEGY }
  }

  return {
    baseDelayMs: serverConfig.reconnect.base_delay_ms ?? DEFAULT_RECONNECT_STRATEGY.baseDelayMs,
    maxDelayMs: serverConfig.reconnect.max_delay_ms ?? DEFAULT_RECONNECT_STRATEGY.maxDelayMs,
    maxAttempts: serverConfig.reconnect.max_attempts ?? DEFAULT_RECONNECT_STRATEGY.maxAttempts,
    multiplier: serverConfig.reconnect.multiplier ?? DEFAULT_RECONNECT_STRATEGY.multiplier,
  }
}

/**
 * 计算重连延迟（指数退避算法）
 * @param attempt 当前重连次数（从1开始）
 * @returns 延迟毫秒数
 */
export function calculateReconnectDelay(attempt: number): number {
  const strategy = getReconnectStrategy()
  const delay = strategy.baseDelayMs * Math.pow(strategy.multiplier, attempt - 1)
  return Math.min(delay, strategy.maxDelayMs)
}

/**
 * 获取心跳配置
 */
export function getHeartbeatConfig(): {
  intervalMs: number
  timeoutMs: number
} {
  if (!serverConfig) {
    return {
      intervalMs: DEFAULT_CONNECTION_CONFIG.heartbeatInterval,
      timeoutMs: DEFAULT_CONNECTION_CONFIG.heartbeatInterval * 2,
    }
  }

  return {
    intervalMs: (serverConfig.websocket.heartbeat_interval_secs ?? 30) * 1000,
    timeoutMs: (serverConfig.websocket.heartbeat_timeout_secs ?? 60) * 1000,
  }
}

// ==================== 状态查询函数 ====================

/**
 * 检查配置是否已初始化
 */
export function isConfigInitialized(): boolean {
  return runtimeState.initialized
}

/**
 * 检查是否成功获取服务端配置
 */
export function hasServerConfig(): boolean {
  return serverConfig !== null
}

/**
 * 获取初始化错误
 */
export function getInitError(): Error | null {
  return runtimeState.initError
}

// ==================== Vue 组合式函数 ====================

/**
 * 在 Vue 组件中使用响应式配置
 * @returns 响应式配置对象
 */
export function useWebSocketConfig(): {
  config: Ref<ClientConfig | null>
  isInitialized: ComputedRef<boolean>
  hasConfig: ComputedRef<boolean>
  connectionConfig: ComputedRef<WebSocketConnectionConfig>
  reconnectStrategy: ComputedRef<ReconnectStrategyConfig>
  heartbeatConfig: ComputedRef<{ intervalMs: number; timeoutMs: number }>
  reload: () => Promise<boolean>
} {
  const config = ref<ClientConfig | null>(serverConfig)

  // 监听配置变更
  onConfigChange((newConfig) => {
    config.value = newConfig
  })

  const isInitialized = computed(() => runtimeState.initialized)
  const hasConfig = computed(() => serverConfig !== null)

  const connectionConfig = computed(() => getConnectionConfig())
  const reconnectStrategy = computed(() => getReconnectStrategy())
  const heartbeatConfig = computed(() => getHeartbeatConfig())

  const reload = async (): Promise<boolean> => {
    return reloadWebSocketConfig()
  }

  return {
    config,
    isInitialized,
    hasConfig,
    connectionConfig,
    reconnectStrategy,
    heartbeatConfig,
    reload,
  }
}

// ==================== 导出默认对象（方便导入）====================

export default {
  init: initWebSocketConfig,
  reload: reloadWebSocketConfig,
  getServerConfig,
  getConnectionConfig,
  getReconnectStrategy,
  getHeartbeatConfig,
  calculateReconnectDelay,
  onConfigChange,
  isInitialized: isConfigInitialized,
  hasServerConfig,
  getInitError,
  useWebSocketConfig,
  DEFAULTS: {
    CONNECTION: DEFAULT_CONNECTION_CONFIG,
    RECONNECT: DEFAULT_RECONNECT_STRATEGY,
  },
}
