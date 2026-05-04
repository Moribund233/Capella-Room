import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getSystemStats } from '@/api/system'

/**
 * 系统状态数据
 */
interface SystemStatusData {
  /** 在线用户数 */
  onlineUsers: number
  /** 活跃房间数 */
  activeRooms: number
  /** WebSocket 连接数 */
  websocketConnections: number
  /** 延迟 (ms) */
  latency: number
  /** 最后更新时间 */
  lastUpdate: Date | null
}

/**
 * 系统状态配置
 */
interface SystemStatusConfig {
  /** 轮询间隔 (ms) */
  interval?: number
  /** 是否自动启动 */
  autoStart?: boolean
}

/**
 * 系统状态状态
 */
const status = ref<SystemStatusData>({
  onlineUsers: 0,
  activeRooms: 0,
  websocketConnections: 0,
  latency: 0,
  lastUpdate: null,
})

const isLoading = ref(false)
const error = ref<Error | null>(null)
let pollInterval: number | null = null
let config: Required<SystemStatusConfig> = {
  interval: 5000,
  autoStart: true,
}

/**
 * 获取系统状态
 */
async function fetchStatus(): Promise<void> {
  const startTime = performance.now()
  isLoading.value = true
  error.value = null

  try {
    const stats = await getSystemStats()
    const endTime = performance.now()

    status.value = {
      onlineUsers: stats.online_users,
      activeRooms: stats.active_rooms,
      websocketConnections: stats.websocket_connections,
      latency: Math.round(endTime - startTime),
      lastUpdate: new Date(),
    }
  } catch (err) {
    error.value = err instanceof Error ? err : new Error('获取系统状态失败')
    console.error('[useSystemStatus] 获取系统状态失败:', err)
  } finally {
    isLoading.value = false
  }
}

/**
 * 开始轮询
 */
function startPolling(): void {
  if (pollInterval) return

  fetchStatus()
  pollInterval = window.setInterval(() => {
    fetchStatus()
  }, config.interval)
}

/**
 * 停止轮询
 */
function stopPolling(): void {
  if (pollInterval) {
    clearInterval(pollInterval)
    pollInterval = null
  }
}

/**
 * 刷新状态
 */
function refresh(): Promise<void> {
  return fetchStatus()
}

/**
 * 系统状态组合式函数
 * 用于获取和轮询系统状态数据（在线用户、活跃房间、延迟等）
 *
 * @example
 * // 基础使用
 * const { status, isLoading, refresh } = useSystemStatus()
 *
 * @example
 * // 自定义配置
 * const { status, startPolling, stopPolling } = useSystemStatus({
 *   interval: 10000,
 *   autoStart: false,
 * })
 *
 * onMounted(() => startPolling())
 * onUnmounted(() => stopPolling())
 */
export function useSystemStatus(userConfig?: SystemStatusConfig) {
  // 合并配置
  config = { ...config, ...userConfig }

  // 自动启动
  onMounted(() => {
    if (config.autoStart) {
      startPolling()
    }
  })

  // 自动停止
  onUnmounted(() => {
    stopPolling()
  })

  return {
    // 状态
    status: computed(() => status.value),
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    // 方法
    refresh,
    startPolling,
    stopPolling,
  }
}

export type { SystemStatusData, SystemStatusConfig }
