export interface Department {
  id: string
  parent_id: string | null
  name: string
  code: string
  sort_order: number
  status: string
  leader: string | null
  created_at: string | null
  updated_at: string | null
}

export interface DepartmentTreeNode {
  id: string
  parent_id: string | null
  name: string
  code: string
  sort_order: number
  status: string
  leader: string | null
  children: DepartmentTreeNode[]
}

export interface CreateDepartmentRequest {
  name: string
  code: string
  parent_id?: string | null
  sort_order?: number
  status?: string
  leader?: string
}

export interface UpdateDepartmentRequest {
  name?: string
  sort_order?: number
  status?: string
  leader?: string
}
