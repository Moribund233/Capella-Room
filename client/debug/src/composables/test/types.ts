/**
 * 测试工具类型定义
 */

/**
 * HTTP 方法类型
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'

/**
 * API 端点定义
 */
export interface ApiEndpoint {
  /** 端点标识 */
  key: string
  /** 显示标签 */
  label: string
  /** HTTP 方法 */
  method: HttpMethod
  /** API 路径（可包含参数占位符如 :id） */
  path: string
  /** 所属分类 */
  category: string
  /** 是否需要认证 */
  requiresAuth: boolean
  /** 请求体示例 */
  bodyExample?: Record<string, unknown>
  /** 路径参数说明 */
  pathParams?: { name: string; description: string; required: boolean }[]
  /** 查询参数说明 */
  queryParams?: { name: string; description: string; required: boolean; default?: string | number }[]
}

/**
 * 请求历史记录项
 */
export interface RequestHistoryItem {
  /** 唯一标识 */
  id: string
  /** 请求时间 */
  timestamp: number
  /** 端点信息 */
  endpoint: ApiEndpoint
  /** 实际请求路径 */
  url: string
  /** 请求头 */
  headers: Record<string, string>
  /** 请求体 */
  body?: string
  /** 响应状态码 */
  statusCode?: number
  /** 响应数据 */
  response?: unknown
  /** 响应时间(ms) */
  duration?: number
  /** 是否出错 */
  error?: boolean
  /** 错误信息 */
  errorMessage?: string
}

/**
 * API 响应结果
 */
export interface ApiTestResult {
  /** 是否成功 */
  success: boolean
  /** 状态码 */
  statusCode: number
  /** 状态文本 */
  statusText: string
  /** 响应头 */
  headers: Record<string, string>
  /** 响应数据 */
  data: unknown
  /** 响应时间(ms) */
  duration: number
}
