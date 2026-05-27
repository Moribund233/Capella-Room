export interface ApiResponse<T> {
  success: boolean
  data?: T
  code?: string
  error?: string
  message?: string
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  has_more: boolean
}

export interface PaginationParams {
  limit?: number
  offset?: number
  before?: string
}
