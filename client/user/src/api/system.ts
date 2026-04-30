/**
 * 系统相关 API
 */

import { apiClient, API_BASE_URL } from './client'
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
 * 健康检查详细响应（后端直接返回，无 ApiResponse 包装）
 */
interface HealthDetailResponse {
  success: boolean
  data: {
    status: string
    timestamp: string
    components: {
      database: {
        status: string
      }
      websocket: {
        status: string
        connections: number
        online_users: number
      }
    }
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
 */
export async function healthCheck(): Promise<{ status: string; version: string }> {
  const response = await apiClient.get<{ status: string; version: string }>('/api/v1/health')
  return response.data
}

/**
 * 获取系统统计信息
 * 使用 /health/detail 端点获取在线用户、连接数等信息
 * 注意：此端点直接返回数据，无 ApiResponse 包装
 */
export async function getSystemStats(): Promise<SystemStats> {
  // 健康检查端点直接返回，不使用 apiClient 的包装处理
  const baseUrl = API_BASE_URL || ''
  const response = await fetch(`${baseUrl}/health/detail`)

  if (!response.ok) {
    throw new Error(`获取系统状态失败: ${response.status}`)
  }

  const data: HealthDetailResponse = await response.json()

  return {
    online_users: data.data.components.websocket.online_users,
    active_rooms: 0, // 健康检查端点不返回房间数
    websocket_connections: data.data.components.websocket.connections,
  }
}
