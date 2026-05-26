export interface MenuItem {
  id: string
  parent_id: string | null
  title_key: string
  title_label: string
  path: string
  icon: string | null
  sort_order: number
  permission_id: string | null
  status: string
  created_at: string | null
  updated_at: string | null
}

export interface MenuTreeNode {
  id: string
  title_key: string
  title_label: string
  path: string
  icon: string | null
  sort_order: number
  children: MenuTreeNode[]
}

export interface CreateMenuRequest {
  parent_id?: string | null
  title_key: string
  title_label: string
  path: string
  icon?: string | null
  sort_order?: number
  permission_id?: string | null
}

export interface UpdateMenuRequest {
  parent_id?: string | null
  title_key?: string
  title_label?: string
  path?: string
  icon?: string | null
  sort_order?: number
  permission_id?: string | null
  status?: string
}

export interface MenuListResponse {
  data: MenuItem[]
}
