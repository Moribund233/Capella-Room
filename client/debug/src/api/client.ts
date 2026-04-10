/**
 * API Client - HTTP 请求封装
 */

import type { ApiResponse } from '@/types/api'
import { getAccessToken } from './token'

// API 基础配置
const API_BASE_URL = import.meta.env.VITE_BACKEND_URL

// 请求配置接口
interface RequestConfig extends RequestInit {
  params?: Record<string, string>
}

/**
 * 构建 URL
 */
function buildUrl(endpoint: string, params?: Record<string, string>): string {
  const url = new URL(endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`)

  if (params) {
    Object.entries(params).forEach(([key, value]) => {
      url.searchParams.append(key, value)
    })
  }

  return url.toString()
}

/**
 * 发送 HTTP 请求
 * 注意：此函数不处理 token 刷新逻辑，只负责发送请求
 */
async function request<T>(endpoint: string, config: RequestConfig = {}): Promise<ApiResponse<T>> {
  const { params, ...fetchConfig } = config
  const url = buildUrl(endpoint, params)

  // 默认请求头
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }

  // 合并自定义请求头
  if (fetchConfig.headers) {
    const customHeaders = fetchConfig.headers as Record<string, string>
    Object.assign(headers, customHeaders)
  }

  // 添加认证头（从 token 模块获取，但不处理刷新逻辑）
  const token = getAccessToken()
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  try {
    const response = await fetch(url, {
      ...fetchConfig,
      headers,
    })

    // 获取响应文本
    const responseText = await response.text()

    // 尝试解析 JSON
    let data: ApiResponse<T>
    try {
      data = JSON.parse(responseText)
    } catch {
      // 如果不是 JSON，构造一个错误响应
      throw new Error(responseText || `HTTP ${response.status}`)
    }

    if (!response.ok) {
      throw new Error(data.message || `HTTP ${response.status}`)
    }

    return data
  } catch (error) {
    if (error instanceof Error) {
      throw error
    }
    throw new Error('Network error')
  }
}

// API Client 对象
export const apiClient = {
  /**
   * GET 请求
   */
  get<T>(endpoint: string, params?: Record<string, string>): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { method: 'GET', params })
  },

  /**
   * POST 请求
   */
  post<T>(endpoint: string, body?: unknown): Promise<ApiResponse<T>> {
    return request<T>(endpoint, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    })
  },

  /**
   * PUT 请求
   */
  put<T>(endpoint: string, body?: unknown): Promise<ApiResponse<T>> {
    return request<T>(endpoint, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    })
  },

  /**
   * DELETE 请求
   */
  delete<T>(endpoint: string): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { method: 'DELETE' })
  },

  /**
   * PATCH 请求
   */
  patch<T>(endpoint: string, body?: unknown): Promise<ApiResponse<T>> {
    return request<T>(endpoint, {
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined,
    })
  },
}

export default apiClient
