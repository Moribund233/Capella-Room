import { ref, computed } from 'vue'
import { getAccessToken } from '@/api/token'
import type { ApiEndpoint, ApiTestResult, RequestHistoryItem } from './types'

export type { ApiEndpoint, ApiTestResult, HttpMethod, RequestHistoryItem } from './types'

/**
 * 预设 API 端点列表
 * 基于实际后端 API 文档
 */
export const apiEndpoints: ApiEndpoint[] = [
  // ========== 认证 API（无需认证）==========
  {
    key: 'auth-register',
    label: '用户注册',
    method: 'POST',
    path: '/api/v1/auth/register',
    category: '认证',
    requiresAuth: false,
    bodyExample: {
      username: 'testuser',
      email: 'test@example.com',
      password: 'SecurePass123!'
    }
  },
  {
    key: 'auth-login',
    label: '用户登录',
    method: 'POST',
    path: '/api/v1/auth/login',
    category: '认证',
    requiresAuth: false,
    bodyExample: {
      email: 'test@example.com',
      password: 'SecurePass123!'
    }
  },
  {
    key: 'auth-refresh',
    label: '刷新 Token',
    method: 'POST',
    path: '/api/v1/auth/refresh',
    category: '认证',
    requiresAuth: false,
    bodyExample: {
      refresh_token: 'your_refresh_token_here'
    }
  },

  // ========== 用户 API（需要认证）==========
  {
    key: 'users-me',
    label: '获取当前用户',
    method: 'GET',
    path: '/api/v1/users/me',
    category: '用户',
    requiresAuth: true
  },
  {
    key: 'users-me-update',
    label: '更新当前用户',
    method: 'PUT',
    path: '/api/v1/users/me',
    category: '用户',
    requiresAuth: true,
    bodyExample: {
      username: 'newusername',
      avatar_url: 'https://example.com/avatar.jpg'
    }
  },
  {
    key: 'users-me-password',
    label: '修改密码',
    method: 'PUT',
    path: '/api/v1/users/me/password',
    category: '用户',
    requiresAuth: true,
    bodyExample: {
      old_password: 'OldPass123!',
      new_password: 'NewPass456!'
    }
  },
  {
    key: 'users-me-rooms',
    label: '获取我的聊天室',
    method: 'GET',
    path: '/api/v1/users/me/rooms',
    category: '用户',
    requiresAuth: true
  },
  {
    key: 'users-logout',
    label: '用户登出',
    method: 'POST',
    path: '/api/v1/users/logout',
    category: '用户',
    requiresAuth: true
  },
  {
    key: 'users-list',
    label: '获取用户列表',
    method: 'GET',
    path: '/api/v1/users',
    category: '用户',
    requiresAuth: true,
    queryParams: [
      { name: 'limit', description: '每页数量', required: false, default: 20 },
      { name: 'offset', description: '偏移量', required: false, default: 0 }
    ]
  },
  {
    key: 'users-detail',
    label: '获取指定用户',
    method: 'GET',
    path: '/api/v1/users/:user_id',
    category: '用户',
    requiresAuth: true,
    pathParams: [
      { name: 'user_id', description: '用户ID', required: true }
    ]
  },

  // ========== 房间 API（需要认证）==========
  {
    key: 'rooms-list',
    label: '获取房间列表',
    method: 'GET',
    path: '/api/v1/rooms',
    category: '房间',
    requiresAuth: true,
    queryParams: [
      { name: 'search', description: '搜索关键词', required: false },
      { name: 'limit', description: '每页数量', required: false, default: 20 },
      { name: 'offset', description: '偏移量', required: false, default: 0 }
    ]
  },
  {
    key: 'rooms-recent',
    label: '获取最近房间',
    method: 'GET',
    path: '/api/v1/rooms/recent',
    category: '房间',
    requiresAuth: true,
    queryParams: [
      { name: 'limit', description: '每页数量', required: false, default: 20 },
      { name: 'offset', description: '偏移量', required: false, default: 0 }
    ]
  },
  {
    key: 'rooms-create',
    label: '创建房间',
    method: 'POST',
    path: '/api/v1/rooms',
    category: '房间',
    requiresAuth: true,
    bodyExample: {
      name: '技术交流群',
      description: '讨论各种技术话题',
      is_private: false,
      max_members: 100
    }
  },
  {
    key: 'rooms-detail',
    label: '获取房间详情',
    method: 'GET',
    path: '/api/v1/rooms/:room_id',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ]
  },
  {
    key: 'rooms-update',
    label: '更新房间信息',
    method: 'PUT',
    path: '/api/v1/rooms/:room_id',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ],
    bodyExample: {
      name: '新的房间名',
      description: '新的描述',
      is_private: true,
      max_members: 50
    }
  },
  {
    key: 'rooms-delete',
    label: '删除房间',
    method: 'DELETE',
    path: '/api/v1/rooms/:room_id',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ]
  },
  {
    key: 'rooms-join',
    label: '加入房间',
    method: 'POST',
    path: '/api/v1/rooms/:room_id/join',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ]
  },
  {
    key: 'rooms-leave',
    label: '离开房间',
    method: 'DELETE',
    path: '/api/v1/rooms/:room_id/leave',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ]
  },
  {
    key: 'rooms-members',
    label: '获取成员列表',
    method: 'GET',
    path: '/api/v1/rooms/:room_id/members',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ]
  },
  {
    key: 'rooms-members-kick',
    label: '踢出成员',
    method: 'DELETE',
    path: '/api/v1/rooms/:room_id/members/:user_id',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true },
      { name: 'user_id', description: '用户ID', required: true }
    ]
  },
  {
    key: 'rooms-members-role',
    label: '设置成员角色',
    method: 'PUT',
    path: '/api/v1/rooms/:room_id/members/:user_id/role',
    category: '房间',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true },
      { name: 'user_id', description: '用户ID', required: true }
    ],
    bodyExample: {
      role: 'admin'
    }
  },
  {
    key: 'rooms-messages',
    label: '获取房间消息',
    method: 'GET',
    path: '/api/v1/rooms/:room_id/messages',
    category: '消息',
    requiresAuth: true,
    pathParams: [
      { name: 'room_id', description: '房间ID', required: true }
    ],
    queryParams: [
      { name: 'limit', description: '每页数量', required: false, default: 50 },
      { name: 'before', description: '游标（消息ID）', required: false }
    ]
  },

  // ========== 消息 API（需要认证）==========
  {
    key: 'messages-search',
    label: '搜索消息',
    method: 'GET',
    path: '/api/v1/messages/search',
    category: '消息',
    requiresAuth: true,
    queryParams: [
      { name: 'q', description: '搜索关键词', required: true },
      { name: 'room_id', description: '限定房间ID', required: false },
      { name: 'limit', description: '结果数量', required: false, default: 50 }
    ]
  },
  {
    key: 'messages-edit',
    label: '编辑消息',
    method: 'PUT',
    path: '/api/v1/messages/:message_id',
    category: '消息',
    requiresAuth: true,
    pathParams: [
      { name: 'message_id', description: '消息ID', required: true }
    ],
    bodyExample: {
      content: '编辑后的消息内容'
    }
  },
  {
    key: 'messages-delete',
    label: '删除消息',
    method: 'DELETE',
    path: '/api/v1/messages/:message_id',
    category: '消息',
    requiresAuth: true,
    pathParams: [
      { name: 'message_id', description: '消息ID', required: true }
    ]
  },
  {
    key: 'messages-history',
    label: '获取消息编辑历史',
    method: 'GET',
    path: '/api/v1/messages/:message_id/history',
    category: '消息',
    requiresAuth: true,
    pathParams: [
      { name: 'message_id', description: '消息ID', required: true }
    ]
  }
]

/**
 * API 测试组合式函数
 *
 * @example
 * const {
 *   endpoints,
 *   categories,
 *   selectedEndpoint,
 *   pathParams,
 *   queryParams,
 *   requestBody,
 *   customHeaders,
 *   history,
 *   lastResult,
 *   loading,
 *   selectEndpoint,
 *   sendRequest,
 *   clearHistory,
 *   loadFromHistory
 * } = useApiTest()
 */
export function useApiTest() {
  // ========== 状态 ==========

  /** 当前选中的端点 */
  const selectedEndpoint = ref<ApiEndpoint | null>(null)

  /** 路径参数值 */
  const pathParams = ref<Record<string, string>>({})

  /** 查询参数值 */
  const queryParams = ref<Record<string, string>>({})

  /** 请求体（JSON 字符串） */
  const requestBody = ref('')

  /** 自定义请求头 */
  const customHeaders = ref<Record<string, string>>({})

  /** 请求历史记录 */
  const history = ref<RequestHistoryItem[]>([])

  /** 最后一次请求结果 */
  const lastResult = ref<ApiTestResult | null>(null)

  /** 加载状态 */
  const loading = ref(false)

  // ========== 计算属性 ==========

  /** 所有端点 */
  const endpoints = computed(() => apiEndpoints)

  /** 所有分类 */
  const categories = computed(() => {
    const cats = new Set(apiEndpoints.map(e => e.category))
    return Array.from(cats)
  })

  /** 按分类分组的端点 */
  const groupedEndpoints = computed<Record<string, ApiEndpoint[]>>(() => {
    const groups: Record<string, ApiEndpoint[]> = {}
    for (const endpoint of apiEndpoints) {
      const category = endpoint.category
      if (!groups[category]) {
        groups[category] = []
      }
      groups[category].push(endpoint)
    }
    return groups
  })

  /** 当前端点的完整 URL */
  const fullUrl = computed(() => {
    if (!selectedEndpoint.value) return ''

    let url = selectedEndpoint.value.path

    // 替换路径参数
    for (const [key, value] of Object.entries(pathParams.value)) {
      url = url.replace(`:${key}`, encodeURIComponent(value))
    }

    // 添加查询参数
    const query = new URLSearchParams()
    for (const [key, value] of Object.entries(queryParams.value)) {
      if (value) {
        query.append(key, value)
      }
    }
    const queryString = query.toString()
    if (queryString) {
      url += `?${queryString}`
    }

    return url
  })

  /** 请求头（包含认证头） */
  const requestHeaders = computed(() => {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...customHeaders.value
    }

    // 如果需要认证，添加 Authorization 头
    const token = getAccessToken()
    if (selectedEndpoint.value?.requiresAuth && token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    return headers
  })

  // ========== 方法 ==========

  /**
   * 选择端点
   * @param endpoint 端点定义
   */
  function selectEndpoint(endpoint: ApiEndpoint): void {
    selectedEndpoint.value = endpoint

    // 初始化路径参数
    pathParams.value = {}
    if (endpoint.pathParams) {
      for (const param of endpoint.pathParams) {
        pathParams.value[param.name] = ''
      }
    }

    // 初始化查询参数
    queryParams.value = {}
    if (endpoint.queryParams) {
      for (const param of endpoint.queryParams) {
        queryParams.value[param.name] = param.default?.toString() || ''
      }
    }

    // 设置请求体示例
    if (endpoint.bodyExample) {
      requestBody.value = JSON.stringify(endpoint.bodyExample, null, 2)
    } else {
      requestBody.value = ''
    }

    // 清空上次结果
    lastResult.value = null
  }

  /**
   * 发送请求
   */
  async function sendRequest(): Promise<ApiTestResult | null> {
    if (!selectedEndpoint.value) return null

    loading.value = true
    const startTime = performance.now()

    try {
      const url = fullUrl.value
      const headers = requestHeaders.value

      // 解析请求体
      let body: unknown = undefined
      if (requestBody.value.trim()) {
        try {
          body = JSON.parse(requestBody.value)
        } catch {
          throw new Error('请求体 JSON 格式错误')
        }
      }

      // 构建 fetch 选项
      const options: RequestInit = {
        method: selectedEndpoint.value.method,
        headers,
        credentials: 'include'
      }

      if (body && ['POST', 'PUT', 'PATCH'].includes(selectedEndpoint.value.method)) {
        options.body = JSON.stringify(body)
      }

      // 发送请求
      const response = await fetch(url, options)
      const duration = Math.round(performance.now() - startTime)

      // 解析响应
      let data: unknown
      const contentType = response.headers.get('content-type')
      if (contentType?.includes('application/json')) {
        data = await response.json()
      } else {
        data = await response.text()
      }

      // 构建响应头对象
      const responseHeaders: Record<string, string> = {}
      response.headers.forEach((value, key) => {
        responseHeaders[key] = value
      })

      const result: ApiTestResult = {
        success: response.ok,
        statusCode: response.status,
        statusText: response.statusText,
        headers: responseHeaders,
        data,
        duration
      }

      lastResult.value = result

      // 添加到历史记录
      addToHistory(result)

      return result
    } catch (error) {
      const duration = Math.round(performance.now() - startTime)
      const errorResult: ApiTestResult = {
        success: false,
        statusCode: 0,
        statusText: 'Error',
        headers: {},
        data: error instanceof Error ? error.message : '未知错误',
        duration
      }

      lastResult.value = errorResult
      addToHistory(errorResult)

      return errorResult
    } finally {
      loading.value = false
    }
  }

  /**
   * 添加到历史记录
   * @param result 请求结果
   */
  function addToHistory(result: ApiTestResult): void {
    if (!selectedEndpoint.value) return

    const historyItem: RequestHistoryItem = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      timestamp: Date.now(),
      endpoint: selectedEndpoint.value,
      url: fullUrl.value,
      headers: { ...requestHeaders.value },
      body: requestBody.value,
      statusCode: result.statusCode,
      response: result.data,
      duration: result.duration,
      error: !result.success,
      errorMessage: !result.success && typeof result.data === 'string' ? result.data : undefined
    }

    history.value.unshift(historyItem)

    // 限制历史记录数量
    if (history.value.length > 20) {
      history.value = history.value.slice(0, 20)
    }
  }

  /**
   * 清空历史记录
   */
  function clearHistory(): void {
    history.value = []
  }

  /**
   * 从历史记录加载
   * @param item 历史记录项
   */
  function loadFromHistory(item: RequestHistoryItem): void {
    selectEndpoint(item.endpoint)
    pathParams.value = { ...extractPathParams(item.url, item.endpoint.path) }
    queryParams.value = { ...extractQueryParams(item.url) }
    requestBody.value = item.body || ''
    customHeaders.value = { ...item.headers }
  }

  /**
   * 从 URL 提取路径参数
   */
  function extractPathParams(url: string, pathTemplate: string): Record<string, string> {
    const params: Record<string, string> = {}
    const urlParts = url.split('?')[0]?.split('/') ?? []
    const templateParts = pathTemplate.split('/')

    for (let i = 0; i < templateParts.length; i++) {
      const templatePart = templateParts[i]
      if (templatePart && templatePart.startsWith(':')) {
        const paramName = templatePart.slice(1)
        params[paramName] = decodeURIComponent(urlParts[i] ?? '')
      }
    }

    return params
  }

  /**
   * 从 URL 提取查询参数
   */
  function extractQueryParams(url: string): Record<string, string> {
    const params: Record<string, string> = {}
    const queryIndex = url.indexOf('?')

    if (queryIndex === -1) return params

    const queryString = url.slice(queryIndex + 1)
    const searchParams = new URLSearchParams(queryString)

    searchParams.forEach((value, key) => {
      params[key] = value
    })

    return params
  }

  /**
   * 格式化 JSON
   * @param data 数据
   */
  function formatJson(data: unknown): string {
    try {
      return JSON.stringify(data, null, 2)
    } catch {
      return String(data)
    }
  }

  return {
    // 状态
    endpoints,
    categories,
    groupedEndpoints,
    selectedEndpoint,
    pathParams,
    queryParams,
    requestBody,
    customHeaders,
    history,
    lastResult,
    loading,
    fullUrl,
    requestHeaders,

    // 方法
    selectEndpoint,
    sendRequest,
    clearHistory,
    loadFromHistory,
    formatJson
  }
}
