import { http } from './request'
import type { ApiResponse } from '@/types'

// ==================== IP安全类型 ====================

/**
 * IP列表类型
 */
export type IPListType = 'blacklist' | 'whitelist'

/**
 * IP条目信息
 */
export interface IPEntry {
  /** IP条目ID */
  id: string
  /** IP地址 */
  ip_address: string
  /** 列表类型 */
  list_type: IPListType
  /** 备注 */
  remark: string | null
  /** 过期时间 (null表示永不过期) */
  expires_at: string | null
  /** 创建时间 */
  created_at: string
  /** 创建者 */
  created_by: string
}

/**
 * IP列表查询参数
 */
export interface IPListParams {
  /** 页码 */
  page?: number
  /** 每页数量 */
  page_size?: number
  /** IP地址搜索 */
  ip_address?: string
  /** 列表类型过滤 */
  list_type?: IPListType
}

/**
 * IP列表响应数据
 */
export interface IPListData {
  /** IP条目列表 */
  items: IPEntry[]
  /** 总数量 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页数量 */
  page_size: number
}

/**
 * 添加IP请求
 */
export interface AddIPRequest {
  /** IP地址 */
  ip_address: string
  /** 列表类型 */
  list_type: IPListType
  /** 备注 */
  remark?: string
  /** 过期时间 (ISO 8601格式, null表示永不过期) */
  expires_at?: string | null
}

/**
 * 批量添加IP请求
 */
export interface BatchAddIPRequest {
  /** IP地址列表 */
  ip_addresses: string[]
  /** 列表类型 */
  list_type: IPListType
  /** 备注 */
  remark?: string
  /** 过期时间 */
  expires_at?: string | null
}

/**
 * 更新IP条目请求
 */
export interface UpdateIPRequest {
  /** 备注 */
  remark?: string
  /** 过期时间 */
  expires_at?: string | null
}

/**
 * 白名单模式状态
 */
export interface WhitelistModeStatus {
  /** 是否启用白名单模式 */
  enabled: boolean
  /** 更新时间 */
  updated_at: string
  /** 更新者 */
  updated_by: string
}

/**
 * 设置白名单模式请求
 */
export interface SetWhitelistModeRequest {
  /** 是否启用 */
  enabled: boolean
}

// ==================== IP安全API ====================

/**
 * IP安全管理API
 */
export const securityApi = {
  /**
   * 获取IP列表
   * @param params 查询参数
   * @returns IP列表数据
   */
  getIPList: (params: IPListParams = {}) => {
    const queryParams = new URLSearchParams()
    if (params.page) queryParams.append('page', String(params.page))
    if (params.page_size) queryParams.append('page_size', String(params.page_size))
    if (params.ip_address) queryParams.append('ip_address', params.ip_address)
    if (params.list_type) queryParams.append('list_type', params.list_type)

    const query = queryParams.toString()
    const url = query ? `/admin/security/ip-list?${query}` : '/admin/security/ip-list'

    return http.get<IPListData>(url)
  },

  /**
   * 添加IP到列表
   * @param data 添加IP请求数据
   * @returns 创建的IP条目
   */
  addIP: (data: AddIPRequest): Promise<ApiResponse<IPEntry>> =>
    http.post<IPEntry>('/admin/security/ip-list', data),

  /**
   * 批量添加IP
   * @param data 批量添加请求数据
   * @returns 创建的IP条目列表
   */
  batchAddIP: (data: BatchAddIPRequest): Promise<ApiResponse<IPEntry[]>> =>
    http.post<IPEntry[]>('/admin/security/ip-list/batch', data),

  /**
   * 更新IP条目
   * @param id IP条目ID
   * @param data 更新数据
   * @returns 更新后的IP条目
   */
  updateIP: (id: string, data: UpdateIPRequest): Promise<ApiResponse<IPEntry>> =>
    http.put<IPEntry>(`/admin/security/ip-list/${id}`, data),

  /**
   * 删除IP条目
   * @param id IP条目ID
   * @returns 操作结果
   */
  deleteIP: (id: string): Promise<ApiResponse<void>> =>
    http.delete<void>(`/admin/security/ip-list/${id}`),

  /**
   * 批量删除IP条目
   * @param ids IP条目ID列表
   * @returns 操作结果
   */
  batchDeleteIP: (ids: string[]): Promise<ApiResponse<void>> =>
    http.post<void>('/admin/security/ip-list/batch-delete', { ids }),

  /**
   * 获取白名单模式状态
   * @returns 白名单模式状态
   */
  getWhitelistMode: (): Promise<ApiResponse<WhitelistModeStatus>> =>
    http.get<WhitelistModeStatus>('/admin/security/whitelist-mode'),

  /**
   * 设置白名单模式
   * @param data 设置请求
   * @returns 更新后的状态
   */
  setWhitelistMode: (data: SetWhitelistModeRequest): Promise<ApiResponse<WhitelistModeStatus>> =>
    http.post<WhitelistModeStatus>('/admin/security/whitelist-mode', data),
}
