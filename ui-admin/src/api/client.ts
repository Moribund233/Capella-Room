/**
 * API Client 模块
 * 封装 HTTP 请求，处理认证、错误处理等
 */

/** API 基础 URL */
const BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

/** 请求配置接口 */
interface RequestConfig extends RequestInit {
  /** 是否跳过认证 */
  skipAuth?: boolean
}

/** API 响应接口 */
export interface ApiResponse<T = unknown> {
  success: boolean
  data?: T
  message?: string
  code?: string
  pagination?: {
    total: number
    limit: number
    offset: number
  }
}

/** API 错误类 */
export class ApiError extends Error {
  constructor(
    message: string,
    public code: string,
    public status: number
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

/**
 * 获取存储的 Token
 * 优先从 localStorage 获取，如果没有则从 sessionStorage 获取
 */
function getToken(): string | null {
  return localStorage.getItem('access_token') || sessionStorage.getItem('access_token')
}

/**
 * 发送 HTTP 请求
 * @param endpoint API 端点（不包含基础 URL）
 * @param config 请求配置
 * @returns API 响应数据
 * @throws ApiError 请求失败时抛出
 */
export async function request<T>(endpoint: string, config: RequestConfig = {}): Promise<ApiResponse<T>> {
  const { skipAuth, ...fetchConfig } = config
  const url = `${BASE_URL}${endpoint}`

  // 设置默认请求头
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((fetchConfig.headers as Record<string, string>) || {}),
  }

  // 添加认证头
  if (!skipAuth) {
    const token = getToken()
    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }
  }

  try {
    const response = await fetch(url, {
      ...fetchConfig,
      headers,
    })

    const data: ApiResponse<T> = await response.json()

    // 处理业务错误
    if (!data.success) {
      throw new ApiError(
        data.message || '请求失败',
        data.code || 'UNKNOWN_ERROR',
        response.status
      )
    }

    return data
  } catch (error) {
    // 网络错误或解析错误
    if (error instanceof ApiError) {
      throw error
    }

    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new ApiError('网络错误，请检查网络连接', 'NETWORK_ERROR', 0)
    }

    throw new ApiError('请求处理失败', 'REQUEST_FAILED', 0)
  }
}

/**
 * GET 请求
 */
export function get<T>(endpoint: string, config?: RequestConfig) {
  return request<T>(endpoint, { ...config, method: 'GET' })
}

/**
 * POST 请求
 */
export function post<T>(endpoint: string, body: unknown, config?: RequestConfig) {
  return request<T>(endpoint, {
    ...config,
    method: 'POST',
    body: JSON.stringify(body),
  })
}

/**
 * PUT 请求
 */
export function put<T>(endpoint: string, body: unknown, config?: RequestConfig) {
  return request<T>(endpoint, {
    ...config,
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

/**
 * DELETE 请求
 */
export function del<T>(endpoint: string, config?: RequestConfig) {
  return request<T>(endpoint, { ...config, method: 'DELETE' })
}
