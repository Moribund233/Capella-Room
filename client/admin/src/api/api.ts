import type { ApiResponse, UserInfo, PaginationResult, PaginationParams } from '@/types'

/**
 * 基础请求配置
 */
const BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api'

/**
 * 通用请求函数
 * @param url 请求地址
 * @param options 请求配置
 * @returns 响应数据
 */
async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  })

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`)
  }

  return response.json()
}

/**
 * 用户相关接口
 */
export const userApi = {
  /**
   * 用户登录
   * @param username 用户名
   * @param password 密码
   * @returns 登录结果
   */
  login(username: string, password: string) {
    return request<ApiResponse<{ token: string; userInfo: UserInfo }>>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    })
  },

  /**
   * 获取当前用户信息
   * @returns 用户信息
   */
  getCurrentUser() {
    return request<ApiResponse<UserInfo>>('/users/me')
  },

  /**
   * 更新用户信息
   * @param data 用户数据
   * @returns 更新结果
   */
  updateUser(data: Partial<UserInfo>) {
    return request<ApiResponse<UserInfo>>('/user/update', {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  },
}

/**
 * 数据相关接口
 */
export const dataApi = {
  /**
   * 获取列表数据
   * @param params 分页参数
   * @returns 分页结果
   */
  getList<T>(params: PaginationParams) {
    return request<ApiResponse<PaginationResult<T>>>(
      `/data/list?page=${params.page}&pageSize=${params.page_size}`,
    )
  },

  /**
   * 获取详情数据
   * @param id 数据ID
   * @returns 详情数据
   */
  getDetail<T>(id: string) {
    return request<ApiResponse<T>>(`/data/detail/${id}`)
  },
}

export { request }
