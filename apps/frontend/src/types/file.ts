export interface FileRecord {
  id: string
  user_id: string
  filename: string
  original_name: string
  mime_type: string
  size: number
  storage_backend: string
  thumbnail_path: string | null
  created_at: string | null
}

export interface FileListResponse {
  data: FileRecord[]
  pagination: {
    page: number
    per_page: number
    total: number
  }
}

export interface FileUploadProgress {
  loaded: number
  total: number
  percent: number
}
