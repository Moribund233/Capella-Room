import { http } from './request'
import type { ApiResponse } from '@/types'

// ==================== Redis 类型 ====================

/**
 * Redis 连接状态（后端实际返回）
 */
export interface RedisStatus {
  /** 是否启用 */
  enabled: boolean
  /** 是否已连接 */
  connected: boolean
  /** 连接池大小 */
  pool_size: number
  /** 活跃连接数 */
  active_connections: number
  /** 空闲连接数 */
  idle_connections: number
  /** 是否集群模式 */
  cluster_mode: boolean
  /** 节点信息 */
  nodes: RedisNodeInfo[]
}

/**
 * Redis 节点信息
 */
export interface RedisNodeInfo {
  /** 节点ID */
  id: string
  /** 连接地址 */
  address: string
  /** 是否已连接 */
  connected: boolean
  /** 延迟（毫秒） */
  latency_ms?: number
}

/**
 * Redis 统计信息（后端实际返回）
 */
export interface RedisStats {
  /** Pub/Sub 频道数 */
  pubsub_channels: number
  /** Pub/Sub 模式数 */
  pubsub_patterns: number
  /** Stream 消息数 */
  stream_messages: number
  /** Stream 消费者数 */
  stream_consumers: number
  /** 使用内存（字节） */
  memory_used: number
  /** 内存峰值（字节） */
  memory_peak: number
  /** 总处理命令数 */
  total_commands_processed: number
  /** 每秒处理命令数 */
  ops_per_second: number
  /** 命中率 */
  hit_rate: number
  /** 运行时间（秒） */
  uptime_seconds: number
}

/**
 * Redis 数据库统计
 */
export interface RedisDbStats {
  /** 数据库编号 */
  db: number
  /** 键数量 */
  keys: number
  /** 过期键数量 */
  expires: number
  /** 平均过期时间 */
  avg_ttl: number
}

/**
 * Redis 键信息
 */
export interface RedisKeyInfo {
  /** 键名 */
  key: string
  /** 类型 */
  type: string
  /** 过期时间（秒） */
  ttl: number
  /** 大小（字节） */
  size: number
}

// ==================== Redis API ====================

/**
 * Redis 管理 API
 */
export const redisApi = {
  /**
   * 获取 Redis 连接状态
   * @returns Redis 连接状态
   */
  getStatus: (): Promise<ApiResponse<RedisStatus>> =>
    http.get<RedisStatus>('/admin/redis/status'),

  /**
   * 获取 Redis 统计信息
   * @returns Redis 统计信息
   */
  getStats: (): Promise<ApiResponse<RedisStats>> =>
    http.get<RedisStats>('/admin/redis/stats'),

  /**
   * 刷新 Redis 连接
   * @returns 操作结果
   */
  refreshConnection: (): Promise<ApiResponse<RedisStatus>> =>
    http.post<RedisStatus>('/admin/redis/refresh', {}),

  /**
   * 获取键列表
   * @param pattern 匹配模式
   * @param count 数量限制
   * @returns 键列表
   */
  getKeys: (pattern?: string, count?: number): Promise<ApiResponse<RedisKeyInfo[]>> => {
    const params = new URLSearchParams()
    if (pattern) params.append('pattern', pattern)
    if (count) params.append('count', String(count))

    const query = params.toString()
    const url = query ? `/admin/redis/keys?${query}` : '/admin/redis/keys'

    return http.get<RedisKeyInfo[]>(url)
  },

  /**
   * 删除键
   * @param key 键名
   * @returns 操作结果
   */
  deleteKey: (key: string): Promise<ApiResponse<void>> =>
    http.delete<void>(`/admin/redis/keys/${encodeURIComponent(key)}`),

  /**
   * 获取键值
   * @param key 键名
   * @returns 键值
   */
  getValue: (key: string): Promise<ApiResponse<unknown>> =>
    http.get<unknown>(`/admin/redis/keys/${encodeURIComponent(key)}/value`),

  /**
   * 设置键过期时间
   * @param key 键名
   * @param seconds 过期时间（秒）
   * @returns 操作结果
   */
  expireKey: (key: string, seconds: number): Promise<ApiResponse<void>> =>
    http.post<void>(`/admin/redis/keys/${encodeURIComponent(key)}/expire`, { seconds }),

  /**
   * 清空当前数据库
   * @returns 操作结果
   */
  flushDb: (): Promise<ApiResponse<void>> =>
    http.post<void>('/admin/redis/flushdb', {}),
}
