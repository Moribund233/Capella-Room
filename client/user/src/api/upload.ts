import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'

export interface UploadResponse {
  id: string
  url: string
  original_name: string
  file_size: number
  mime_type: string
}

/**
 * 文件上传相关 API
 */
export const uploadApi = {
  /**
   * 通用文件上传
   * @param file 文件对象
   * @param usageType 文件用途
   */
  uploadFile(file: File, usageType: 'avatar' | 'message' | 'room_cover' | 'general' = 'general'): Promise<ApiResponse<UploadResponse>> {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('usage_type', usageType)

    return httpClient.post('/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
  },

  /**
   * 上传图片
   * @param file 图片文件
   * @param usageType 文件用途
   */
  uploadImage(file: File, usageType: 'avatar' | 'message' | 'room_cover' | 'general' = 'general'): Promise<ApiResponse<UploadResponse>> {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('usage_type', usageType)

    return httpClient.post('/upload/image', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
  },

  /**
   * 上传头像
   * @param file 头像图片文件
   */
  uploadAvatar(file: File): Promise<ApiResponse<UploadResponse>> {
    const formData = new FormData()
    formData.append('file', file)

    return httpClient.post('/upload/avatar', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
  },

  /**
   * 获取文件列表
   * @param params 查询参数
   */
  getFiles(params?: {
    category?: 'image' | 'document' | 'video' | 'audio' | 'other'
    usage_type?: 'avatar' | 'message' | 'room_cover' | 'general'
    limit?: number
    offset?: number
  }): Promise<ApiResponse<{ files: UploadResponse[]; total: number }>> {
    return httpClient.get('/files', { params })
  },

  /**
   * 获取文件详情
   * @param fileId 文件ID
   */
  getFile(fileId: string): Promise<ApiResponse<UploadResponse>> {
    return httpClient.get(`/files/${fileId}`)
  },

  /**
   * 删除文件
   * @param fileId 文件ID
   */
  deleteFile(fileId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/files/${fileId}`)
  }
}
