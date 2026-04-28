import { http } from './request'
import type { ApiResponse, UserInfo, UserRole, UserStatus } from '@/types'

/**
 * 用户列表查询参数
 */
export interface UserListParams {
  /** 页码 */
  page?: number
  /** 每页数量 */
  page_size?: number
  /** 搜索关键词 */
  search?: string
}

/**
 * 用户列表响应数据
 */
export interface UserListData {
  /** 用户列表 */
  users: UserInfo[]
  /** 总用户数 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页数量 */
  page_size: number
}

/**
 * 修改用户角色请求
 */
export interface UpdateUserRoleRequest {
  /** 新角色 */
  role: UserRole
}

/**
 * 修改用户状态请求
 */
export interface UpdateUserStatusRequest {
  /** 新状态 */
  status: UserStatus
}

/**
 * 重置密码请求
 */
export interface ResetPasswordRequest {
  /** 新密码 */
  password: string
}

/**
 * 管理员相关 API
 */
export const adminApi = {
  /**
   * 获取用户列表
   * @param params 查询参数
   * @returns 用户列表数据
   */
  getUserList(params: UserListParams = {}): Promise<ApiResponse<UserListData>> {
    const queryParams = new URLSearchParams()
    if (params.page) queryParams.append('page', String(params.page))
    if (params.page_size) queryParams.append('page_size', String(params.page_size))
    if (params.search) queryParams.append('search', params.search)

    const query = queryParams.toString()
    return http.get<UserListData>(`/admin/users${query ? `?${query}` : ''}`)
  },

  /**
   * 获取用户详情
   * @param userId 用户ID
   * @returns 用户详情
   */
  getUserDetail(userId: string): Promise<ApiResponse<UserInfo>> {
    return http.get<UserInfo>(`/admin/users/${userId}`)
  },

  /**
   * 修改用户角色
   * @param userId 用户ID
   * @param data 角色数据
   * @returns 更新后的用户信息
   */
  updateUserRole(userId: string, data: UpdateUserRoleRequest): Promise<ApiResponse<UserInfo>> {
    return http.put<UserInfo>(`/admin/users/${userId}/role`, data)
  },

  /**
   * 修改用户状态（禁用/启用）
   * @param userId 用户ID
   * @param data 状态数据
   * @returns 更新后的用户信息
   */
  updateUserStatus(userId: string, data: UpdateUserStatusRequest): Promise<ApiResponse<UserInfo>> {
    return http.put<UserInfo>(`/admin/users/${userId}/status`, data)
  },

  /**
   * 删除用户
   * @param userId 用户ID
   * @returns 操作结果
   */
  deleteUser(userId: string): Promise<ApiResponse<void>> {
    return http.delete<void>(`/admin/users/${userId}`)
  },

  /**
   * 重置用户密码
   * @param userId 用户ID
   * @param data 密码数据
   * @returns 操作结果
   */
  resetUserPassword(userId: string, data: ResetPasswordRequest): Promise<ApiResponse<void>> {
    return http.put<void>(`/admin/users/${userId}/password`, data)
  },
}
