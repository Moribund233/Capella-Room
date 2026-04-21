/**
 * 系统相关 API
 */

import { apiClient } from './client'
import type { ClientConfig } from '@/types/api'

/**
 * 获取客户端配置
 */
export async function getClientConfig(): Promise<ClientConfig> {
  const response = await apiClient.get<ClientConfig>('/api/v1/config')
  return response.data
}

/**
 * 健康检查
 */
export async function healthCheck(): Promise<{ status: string; version: string }> {
  const response = await apiClient.get<{ status: string; version: string }>('/api/v1/health')
  return response.data
}
