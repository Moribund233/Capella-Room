/**
 * 用户管理 API
 * 负责用户的查询、更新、删除等操作
 */

import { apiClient } from './client'
import type { User } from '@/types/api'

// 用户列表响应
export interface UserListResponse {
  users: User[]
  total: number
}

// 更新用户请求
export interface UpdateUserRequest {
  username?: string
  email?: string
  role?: 'user' | 'admin' | 'super_admin'
  status?: 'active' | 'inactive'
}

// 创建用户请求（管理员）
export interface CreateUserRequest {
  username: string
  email: string
  password: string
  role?: 'user' | 'admin'
}

/**
 * 获取用户列表
 * @param params 查询参数
 * @returns 用户列表
 */
export async function getUsers(params?: {
  search?: string
  role?: string
  status?: string
  page?: number
  per_page?: number
}): Promise<User[]> {
  const queryParams: Record<string, string> = {}
  if (params?.search) queryParams.search = params.search
  if (params?.role) queryParams.role = params.role
  if (params?.status) queryParams.status = params.status
  if (params?.page) queryParams.page = String(params.page)
  if (params?.per_page) queryParams.per_page = String(params.per_page)

  const response = await apiClient.get<User[] | UserListResponse>('/api/v1/users', queryParams)
  // 适配两种可能的响应格式：直接返回数组或包装在 users 字段中
  const data = response.data
  if (Array.isArray(data)) {
    return data
  }
  return data.users || []
}

/**
 * 获取用户详情
 * @param userId 用户ID
 * @returns 用户信息
 */
export async function getUserDetail(userId: string): Promise<User> {
  const response = await apiClient.get<User>(`/api/v1/users/${userId}`)
  return response.data
}

/**
 * 更新用户信息
 * @param userId 用户ID
 * @param data 更新信息
 * @returns 更新后的用户信息
 */
export async function updateUser(userId: string, data: UpdateUserRequest): Promise<User> {
  const response = await apiClient.put<User>(`/api/v1/users/${userId}`, data)
  return response.data
}

/**
 * 删除用户
 * @param userId 用户ID
 */
export async function deleteUser(userId: string): Promise<void> {
  await apiClient.delete(`/api/v1/users/${userId}`)
}

/**
 * 创建用户（管理员）
 * @param data 用户信息
 * @returns 创建的用户
 */
export async function createUser(data: CreateUserRequest): Promise<User> {
  const response = await apiClient.post<User>('/api/v1/users', data)
  return response.data
}
