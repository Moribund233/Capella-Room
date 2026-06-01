import { useAuthStore } from '@/store'
import type { ApiResponse } from '@/types'

/**
 * API 基础 URL
 */
const BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1'

/**
 * 请求配置选项
 */
interface RequestOptions extends RequestInit {
  /** 是否需要认证 */
  requireAuth?: boolean
  /** 查询参数 */
  params?: Record<string, unknown>
}

/**
 * 构建完整的请求 URL
 * @param url 请求路径
 * @param params 查询参数
 * @returns 完整 URL
 */
function buildUrl(url: string, params?: Record<string, unknown>): string {
  if (url.startsWith('http')) {
    return url
  }
  const baseUrl = BASE_URL.endsWith('/') ? BASE_URL.slice(0, -1) : BASE_URL
  const path = url.startsWith('/') ? url : `/${url}`
  let fullUrl = `${baseUrl}${path}`

  // 添加查询参数
  if (params && Object.keys(params).length > 0) {
    const searchParams = new URLSearchParams()
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined && value !== null) {
        searchParams.append(key, String(value))
      }
    }
    const queryString = searchParams.toString()
    if (queryString) {
      fullUrl += `?${queryString}`
    }
  }

  return fullUrl
}

/**
 * 获取请求头
 * @param options 请求选项
 * @returns 请求头对象
 */
function getHeaders(options: RequestOptions): Record<string, string> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }

  // 合并原有请求头
  if (options.headers) {
    const existingHeaders = options.headers as Record<string, string>
    Object.assign(headers, existingHeaders)
  }

  // 添加认证令牌
  if (options.requireAuth !== false) {
    const authStore = useAuthStore()
    const token = authStore.accessToken
    if (token) {
      headers.Authorization = `Bearer ${token}`
    }
  }

  return headers
}

/**
 * 处理响应
 * @param response Fetch 响应对象
 * @returns 解析后的数据
 */
async function handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
  const data = await response.json() as ApiResponse<T>

  if (!response.ok) {
    // 处理特定状态码
    if (response.status === 401) {
      const authStore = useAuthStore()
      authStore.clearAuth()
      window.location.href = '/login'
    }
    throw new Error(data.message || `HTTP error! status: ${response.status}`)
  }

  return data
}

/**
 * 通用请求函数
 * @param url 请求地址
 * @param options 请求配置
 * @returns 响应数据
 */
export async function request<T>(url: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
  const { params, ...fetchOptions } = options
  const response = await fetch(buildUrl(url, params), {
    ...fetchOptions,
    headers: getHeaders(fetchOptions),
  })

  return handleResponse<T>(response)
}

/**
 * HTTP 方法封装
 */
export const http = {
  /**
   * GET 请求
   * @param url 请求地址
   * @param options 请求配置
   * @returns 响应数据
   */
  get<T>(url: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return request<T>(url, { ...options, method: 'GET' })
  },

  /**
   * POST 请求
   * @param url 请求地址
   * @param body 请求体
   * @param options 请求配置
   * @returns 响应数据
   */
  post<T>(url: string, body?: unknown, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return request<T>(url, {
      ...options,
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    })
  },

  /**
   * PUT 请求
   * @param url 请求地址
   * @param body 请求体
   * @param options 请求配置
   * @returns 响应数据
   */
  put<T>(url: string, body?: unknown, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return request<T>(url, {
      ...options,
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    })
  },

  /**
   * DELETE 请求
   * @param url 请求地址
   * @param options 请求配置
   * @returns 响应数据
   */
  delete<T>(url: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return request<T>(url, { ...options, method: 'DELETE' })
  },

  /**
   * PATCH 请求
   * @param url 请求地址
   * @param body 请求体
   * @param options 请求配置
   * @returns 响应数据
   */
  patch<T>(url: string, body?: unknown, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return request<T>(url, {
      ...options,
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined,
    })
  },
}

export { buildUrl }
