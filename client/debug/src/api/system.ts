/**
 * 系统状态 API
 * 使用后端现有的健康检查端点和管理员端点
 */

import { apiClient } from './client'
import type { ClientConfig } from '@/types/config'

// 系统状态
export interface SystemStatus {
  status: 'healthy' | 'degraded' | 'unhealthy'
  timestamp: string
}

// 系统统计
export interface SystemStats {
  active_users: number
  total_rooms: number
  total_messages: number
  websocket_connections: number
}

// 连接信息
export interface ConnectionInfo {
  api_url: string
  websocket_url: string
  version: string
}

// 管理员系统统计（更详细）
export interface AdminSystemStats {
  total_users: number
  total_rooms: number
  total_messages: number
  online_users: number
  today_new_users: number
  today_messages: number
}

/**
 * 获取系统状态
 * 使用 /health 端点（公开）
 * @returns 系统状态
 */
export async function getSystemStatus(): Promise<SystemStatus> {
  const response = await apiClient.get<SystemStatus>('/health')
  return response.data
}

/**
 * 获取系统统计
 * 使用 /health/detail 端点获取 WebSocket 连接数等信息（公开）
 * @returns 系统统计数据（基础信息）
 */
export async function getSystemStats(): Promise<SystemStats> {
  const response = await apiClient.get<{
    status: string
    components: {
      websocket: {
        connections: number
        online_users: number
      }
    }
  }>('/health/detail')

  // 适配前端期望的数据格式
  return {
    active_users: response.data.components.websocket.online_users,
    total_rooms: 0, // 健康检查端点不返回房间数
    total_messages: 0, // 健康检查端点不返回消息数
    websocket_connections: response.data.components.websocket.connections
  }
}

/**
 * 获取管理员系统统计
 * 使用 /api/v1/admin/stats 端点（需要管理员权限）
 * @returns 详细的系统统计数据
 */
export async function getAdminSystemStats(): Promise<AdminSystemStats> {
  const response = await apiClient.get<AdminSystemStats>('/api/v1/admin/stats')
  return response.data
}

/**
 * 获取连接信息
 * 使用 /api/version 端点（公开）
 * @returns 连接信息
 */
export async function getConnectionInfo(): Promise<ConnectionInfo> {
  const response = await apiClient.get<{
    version: string
    name: string
  }>('/api/version')

  const baseUrl = import.meta.env.VITE_BACKEND_URL || 'http://localhost:8080'
  const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080'

  return {
    api_url: baseUrl,
    websocket_url: wsUrl,
    version: response.data.version
  }
}

/**
 * 获取客户端配置
 * 使用 /api/config/client 端点（公开）
 * @returns 客户端配置（WebSocket、重连、上传等配置）
 */
export async function getClientConfig(): Promise<ClientConfig> {
  const response = await apiClient.get<ClientConfig>('/api/config/client')
  return response.data
}

// 性能指标
export interface PerformanceMetrics {
  total_messages: number
  total_connections: number
  current_online_users: number
  active_rooms: number
  timestamp: string
}

/**
 * 获取性能指标
 * 使用 /api/v1/admin/stats/performance 端点（需要管理员权限）
 * @returns 实时性能指标数据
 */
export async function getPerformanceMetrics(): Promise<PerformanceMetrics> {
  const response = await apiClient.get<PerformanceMetrics>('/api/v1/admin/stats/performance')
  return response.data
}
