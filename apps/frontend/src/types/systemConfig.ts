export interface SystemConfig {
  id: string
  group_key: string
  config_key: string
  value: string
  value_type: "string" | "number" | "boolean" | "json"
  description: string
  is_public: boolean
  updated_at: string
}
