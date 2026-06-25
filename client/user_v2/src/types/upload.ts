export interface ChunkedInitResponse {
  session_id: string
  chunk_size: number
  total_chunks: number
}

export interface ChunkUploadResponse {
  received: number
  total: number
}

export interface ChunkedSessionStatus {
  session_id: string
  file_name: string
  file_size: number
  mime_type: string
  status: string
  total_chunks: number
  received_chunks: number[]
  missing_chunks: number[]
  created_at: string
}
