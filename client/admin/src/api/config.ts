import { http } from './request'
import type { ApiResponse } from '@/types'

/**
 * 客户端 WebSocket 配置
 */
export interface ClientWebSocketConfig {
  /** 心跳间隔（秒） */
  heartbeat_interval_secs: number
  /** 心跳超时（秒） */
  heartbeat_timeout_secs: number
  /** 认证超时（秒） */
  auth_timeout_secs: number
}

/**
 * 客户端重连配置
 */
export interface ClientReconnectConfig {
  /** 基础延迟（毫秒） */
  base_delay_ms: number
  /** 最大延迟（毫秒） */
  max_delay_ms: number
  /** 最大重连次数 */
  max_attempts: number
  /** 延迟倍数（指数退避） */
  multiplier: number
}

/**
 * 客户端上传配置
 */
export interface ClientUploadConfig {
  /** 最大文件大小（字节） */
  max_file_size: number
  /** 人类可读的最大文件大小（如 "10MB"） */
  max_file_size_human: string
}

/**
 * 客户端系统配置
 */
export interface ClientSystemConfig {
  /** 系统名称 */
  name: string
  /** 系统版本 */
  version: string
  /** 是否处于维护模式 */
  maintenance_mode: boolean
  /** 维护模式提示消息 */
  maintenance_message: string
}

/**
 * 客户端监控配置
 */
export interface ClientMonitorConfig {
  /** 监控数据刷新频率（秒） */
  refresh_interval_secs: number
}

/**
 * 客户端配置
 */
export interface ClientConfig {
  /** WebSocket 配置 */
  websocket: ClientWebSocketConfig
  /** 重连配置 */
  reconnect: ClientReconnectConfig
  /** 上传配置 */
  upload: ClientUploadConfig
  /** 系统状态 */
  system: ClientSystemConfig
  /** 监控配置 */
  monitor: ClientMonitorConfig
}

/**
 * 获取客户端配置
 * @returns 客户端配置
 */
export function getClientConfig(): Promise<ApiResponse<ClientConfig>> {
  // 客户端配置端点不在 /api/v1 下，需要移除 v1 后缀
  const baseUrl = (import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1').replace('/api/v1', '')
  return http.get<ClientConfig>(`${baseUrl}/api/config/client`)
}

/**
 * 获取心跳超时时间（毫秒）
 * 使用服务端配置的 heartbeat_timeout_secs，并留出 80% 的缓冲时间
 * @param config 客户端配置
 * @returns 超时时间（毫秒）
 */
export function getHeartbeatTimeoutMs(config: ClientConfig | null): number {
  const timeoutSecs = config?.websocket?.heartbeat_timeout_secs ?? 90
  // 使用服务端超时时间的 80% 作为客户端检测阈值（留出缓冲）
  return Math.floor(timeoutSecs * 1000 * 0.8)
}
