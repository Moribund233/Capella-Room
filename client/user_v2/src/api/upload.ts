import httpClient from '@/services/http'
import { useConfigStore } from '@/stores/config'
import type { ApiResponse } from '@/types/api'
import type { ChunkedInitResponse, ChunkUploadResponse, ChunkedSessionStatus } from '@/types/upload'

export interface UploadResponse {
  id: string
  url: string
  original_name: string
  file_size: number
  mime_type: string
}

type UsageType = 'avatar' | 'message' | 'room_cover' | 'general'

function toUploadResponse(data: {
  id: string
  file_url?: string
  url?: string
  original_name: string
  file_size: number
  mime_type: string
}): UploadResponse {
  return {
    id: data.id,
    url: data.file_url ?? data.url ?? '',
    original_name: data.original_name,
    file_size: data.file_size,
    mime_type: data.mime_type,
  }
}

export const uploadApi = {
  uploadFile(file: File, usageType: UsageType = 'general'): Promise<ApiResponse<UploadResponse>> {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('usage_type', usageType)
    return httpClient.post('/upload', formData)
  },

  uploadImage(file: File, usageType: UsageType = 'general'): Promise<ApiResponse<UploadResponse>> {
    const formData = new FormData()
    formData.append('file', file)
    formData.append('usage_type', usageType)
    return httpClient.post('/upload/image', formData)
  },

  uploadAvatar(file: File): Promise<ApiResponse<UploadResponse>> {
    const formData = new FormData()
    formData.append('file', file)
    return httpClient.post('/upload/avatar', formData)
  },

  /** 初始化分片上传会话（后端返回裸 JSON，不含 success/data 包装） */
  async initChunkedUpload(fileName: string, fileSize: number, mimeType: string, usageType: UsageType, totalChunks: number): Promise<ChunkedInitResponse> {
    const raw: any = await httpClient.post('/upload/chunked/init', {
      file_name: fileName,
      file_size: fileSize,
      mime_type: mimeType,
      usage_type: usageType,
      total_chunks: totalChunks,
    })
    return {
      session_id: raw.session_id ?? raw.data?.session_id,
      chunk_size: raw.chunk_size ?? raw.data?.chunk_size,
      total_chunks: raw.total_chunks ?? raw.data?.total_chunks,
    }
  },

  /** 上传单个分片（后端返回裸 JSON） */
  async uploadChunk(sessionId: string, chunkIndex: number, chunk: Blob): Promise<ChunkUploadResponse> {
    const formData = new FormData()
    formData.append('chunk', chunk)
    const raw: any = await httpClient.post(`/upload/chunked/${sessionId}/${chunkIndex}`, formData, {
      timeout: 30000,
    })
    return {
      received: raw.received ?? raw.data?.received,
      total: raw.total ?? raw.data?.total,
    }
  },

  /** 查询分片上传状态 */
  getChunkStatus(sessionId: string): Promise<ChunkedSessionStatus> {
    return httpClient.get(`/upload/chunked/${sessionId}/status`) as any
  },

  /** 完成分片上传（后端返回裸 JSON） */
  async completeChunkedUpload(sessionId: string): Promise<UploadResponse> {
    const raw: any = await httpClient.post(`/upload/chunked/${sessionId}/complete`)
    const file = raw.file ?? raw.data?.file ?? raw
    return toUploadResponse(file)
  },

  /** 取消分片上传 */
  cancelChunkedUpload(sessionId: string): Promise<void> {
    return httpClient.delete(`/upload/chunked/${sessionId}`)
  },

  getFiles(params?: {
    category?: 'image' | 'document' | 'video' | 'audio' | 'other'
    usage_type?: UsageType
    limit?: number
    offset?: number
  }): Promise<ApiResponse<{ files: UploadResponse[]; total: number }>> {
    return httpClient.get('/files', { params })
  },

  getFile(fileId: string): Promise<ApiResponse<UploadResponse>> {
    return httpClient.get(`/files/${fileId}`)
  },

  deleteFile(fileId: string): Promise<ApiResponse<void>> {
    return httpClient.delete(`/files/${fileId}`)
  },
}

export interface SmartUploadOptions {
  endpoint: 'image' | 'avatar' | 'file'
  usageType: UsageType
  onProgress?: (progress: number) => void
}

export async function smartUpload(
  file: File,
  options: SmartUploadOptions,
): Promise<ApiResponse<UploadResponse>> {
  const configStore = useConfigStore()
  await configStore.ensureLoaded()
  const { chunked_upload_enabled, default_chunk_size } = configStore.config.upload

  const useChunked = chunked_upload_enabled && file.size > default_chunk_size

  if (!useChunked) {
    switch (options.endpoint) {
      case 'avatar':
        return uploadApi.uploadAvatar(file)
      case 'image':
        return uploadApi.uploadImage(file, options.usageType)
      case 'file':
      default:
        return uploadApi.uploadFile(file, options.usageType)
    }
  }

  const totalChunks = Math.ceil(file.size / default_chunk_size)
  const { session_id, chunk_size } = await uploadApi.initChunkedUpload(
    file.name,
    file.size,
    file.type || 'application/octet-stream',
    options.usageType,
    totalChunks,
  )

  for (let i = 0; i < totalChunks; i++) {
    const start = i * chunk_size
    const end = Math.min(start + chunk_size, file.size)
    const chunk = file.slice(start, end)

    await uploadApi.uploadChunk(session_id, i, chunk)

    if (options.onProgress) {
      options.onProgress(Math.round(((i + 1) / totalChunks) * 100))
    }
  }

  const result = await uploadApi.completeChunkedUpload(session_id)
  return { success: true, data: result } as ApiResponse<UploadResponse>
}
