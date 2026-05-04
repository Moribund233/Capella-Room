/**
 * UI 配置相关 API
 */

import { apiClient } from './client'
import type { UIConfigResponse, SaveUIConfigParams } from '@/types'

/**
 * 获取用户云端 UI 配置
 */
export async function getUserConfig(): Promise<UIConfigResponse> {
  const response = await apiClient.get<UIConfigResponse>('/ui/config')
  return response.data
}

/**
 * 保存用户云端 UI 配置
 */
export async function saveUserConfig(params: SaveUIConfigParams): Promise<void> {
  await apiClient.post('/ui/config', {
    body: JSON.stringify(params),
  })
}

/**
 * 重置用户云端 UI 配置
 */
export async function resetUserConfig(): Promise<void> {
  await apiClient.delete('/ui/config')
}

// UI API 对象
export const uiApi = {
  getUserConfig,
  saveUserConfig,
  resetUserConfig,
}
