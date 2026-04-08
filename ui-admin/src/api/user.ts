/**
 * 用户相关 API
 * 包含个人信息获取、更新、密码修改等功能
 */
import { get, put, type ApiResponse } from './client'
import type { User, UpdateUsernameRequest, ChangePasswordRequest } from '@/types'

export type { UpdateUsernameRequest, ChangePasswordRequest }

/**
 * 获取当前用户信息
 * @returns 当前用户详细信息
 */
export function getCurrentUser(): Promise<ApiResponse<User>> {
  return get<User>('/users/me')
}

/**
 * 更新当前用户信息
 * @param data 更新数据（用户名或头像）
 * @returns 更新后的用户信息
 */
export function updateUser(data: Partial<UpdateUsernameRequest>): Promise<ApiResponse<User>> {
  return put<User>('/users/me', data)
}

/**
 * 修改当前用户密码
 * @param data 密码修改数据
 * @returns 修改结果
 */
export function changePassword(data: ChangePasswordRequest): Promise<ApiResponse<void>> {
  return put<void>('/users/me/password', data)
}
