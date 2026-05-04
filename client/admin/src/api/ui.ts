import { http } from './request'
import type { ApiResponse, UIConfig, SaveUIConfigParams } from '@/types'

/**
 * UI 配置相关接口
 */
export const uiApi = {
  /**
   * 获取用户云端 UI 配置
   * @returns 云端 UI 配置
   */
  getUserConfig(): Promise<ApiResponse<UIConfig>> {
    return http.get<UIConfig>('/ui/config')
  },

  /**
   * 保存用户云端 UI 配置
   * @param params 配置参数
   * @returns 保存结果
   */
  saveUserConfig(params: SaveUIConfigParams): Promise<ApiResponse<void>> {
    return http.post<void>('/ui/config', params)
  },

  /**
   * 重置用户云端 UI 配置为默认
   * @returns 重置结果
   */
  resetUserConfig(): Promise<ApiResponse<void>> {
    return http.delete<void>('/ui/config')
  },

  /**
   * 同步云端配置到本地
   * @returns 最新云端配置
   */
  syncConfig(): Promise<ApiResponse<UIConfig>> {
    return http.get<UIConfig>('/ui/config/sync')
  },
}
