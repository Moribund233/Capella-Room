/**
 * 系统相关 API
 */

import { API_BASE_URL } from './client'
import type { ClientConfig } from '@/types/api'

/**
 * 系统统计信息
 */
export interface SystemStats {
  /** 在线用户数 */
  online_users: number
  /** 活跃房间数 */
  active_rooms: number
  /** WebSocket 连接数 */
  websocket_connections: number
  /** 总消息数 */
  total_messages?: number
}

/**
 * 健康检查响应
 */
interface HealthResponse {
  success: boolean
  data: {
    status: string
    timestamp: string
  }
}

/**
 * 获取客户端配置
 */
export async function getClientConfig(): Promise<ClientConfig> {
  // 客户端配置端点不在 /api/v1 下，直接使用 /api/config/client
  const response = await fetch('/api/config/client')

  if (!response.ok) {
    throw new Error(`获取客户端配置失败: ${response.status}`)
  }

  const data: { success: boolean; data: ClientConfig; message?: string } = await response.json()

  if (!data.success) {
    throw new Error(data.message || '获取客户端配置失败')
  }

  return data.data
}

/**
 * 健康检查
 * 使用 /health 端点，无需认证
 */
export async function healthCheck(): Promise<{ status: string; version: string }> {
  // 健康检查端点不使用 /api/v1 前缀
  const baseUrl = API_BASE_URL?.replace('/api/v1', '') || ''
  const response = await fetch(`${baseUrl}/health`)

  if (!response.ok) {
    throw new Error(`健康检查失败: ${response.status}`)
  }

  const data: HealthResponse = await response.json()
  return {
    status: data.data.status,
    version: '1.0.0', // 健康检查端点不返回版本号
  }
}

/**
 * 获取系统统计信息
 * 注意：后端可能未实现 /health/detail 端点，暂时使用基础健康检查
 */
export async function getSystemStats(): Promise<SystemStats> {
  // 尝试使用 /health/detail 端点
  const baseUrl = API_BASE_URL?.replace('/api/v1', '') || ''

  try {
    const response = await fetch(`${baseUrl}/health/detail`)

    if (response.ok) {
      const data = await response.json()
      return {
        online_users: data.data?.components?.websocket?.online_users || 0,
        active_rooms: 0,
        websocket_connections: data.data?.components?.websocket?.connections || 0,
      }
    }
  } catch {
    // /health/detail 端点不可用，降级处理
  }

  // 降级：返回基础健康状态
  try {
    const response = await fetch(`${baseUrl}/health`)
    if (response.ok) {
      return {
        online_users: 0,
        active_rooms: 0,
        websocket_connections: 0,
      }
    }
  } catch {
    // 健康检查也失败
  }

  // 如果都失败了，返回空数据
  return {
    online_users: 0,
    active_rooms: 0,
    websocket_connections: 0,
  }
}
