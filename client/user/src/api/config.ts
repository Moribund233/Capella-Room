import type { ApiResponse } from '@/types/api'
import type { ClientConfig } from '@/types/config'

/** 客户端配置 API（公开端点，无需认证） */
export const configApi = {
  /**
   * 获取客户端配置
   * 注意：该端点位于 /api/config/client（不在 /api/v1 下）
   */
  async getClientConfig(): Promise<ApiResponse<ClientConfig>> {
    const baseUrl = import.meta.env.VITE_API_BASE_URL
    // 从 base URL 中提取 origin（去掉 path 部分），因为 config 端点不在 v1 命名空间下
    const origin = baseUrl ? new URL(baseUrl).origin : ''
    const url = `${origin}/api/config/client`
    const res = await fetch(url)
    return res.json()
  },
}
