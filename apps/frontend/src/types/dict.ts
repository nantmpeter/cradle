export interface DictType {
  id: string
  name: string
  code: string
  status: string
  remark: string | null
  created_at: string | null
  updated_at: string | null
}

export interface DictItem {
  id: string
  dict_type_id: string
  label: string
  value: string
  sort_order: number
  status: string
  remark: string | null
  created_at: string | null
  updated_at: string | null
}

export interface DictPublicItem {
  label: string
  value: string
}

export interface CreateDictTypeRequest {
  name: string
  code: string
  status?: string
  remark?: string
}

export interface UpdateDictTypeRequest {
  name?: string
  status?: string
  remark?: string
}

export interface CreateDictItemRequest {
  label: string
  value: string
  sort_order?: number
  status?: string
  remark?: string
}

export interface UpdateDictItemRequest {
  label?: string
  value?: string
  sort_order?: number
  status?: string
  remark?: string
}

export interface DictTypeListParams {
  search?: string
  status?: string
  page?: number
  per_page?: number
}
