use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 字典类型数据库模型
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DictType {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// 字典项数据库模型
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DictItem {
    pub id: Uuid,
    pub dict_type_id: Uuid,
    pub label: String,
    pub value: String,
    pub sort_order: i32,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// 字典类型响应 DTO
#[derive(Debug, Serialize)]
pub struct DictTypeResponse {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<DictType> for DictTypeResponse {
    fn from(dt: DictType) -> Self {
        Self {
            id: dt.id,
            name: dt.name,
            code: dt.code,
            status: dt.status,
            remark: dt.remark,
            created_at: dt.created_at,
            updated_at: dt.updated_at,
        }
    }
}

/// 字典项响应 DTO
#[derive(Debug, Serialize)]
pub struct DictItemResponse {
    pub id: Uuid,
    pub dict_type_id: Uuid,
    pub label: String,
    pub value: String,
    pub sort_order: i32,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<DictItem> for DictItemResponse {
    fn from(di: DictItem) -> Self {
        Self {
            id: di.id,
            dict_type_id: di.dict_type_id,
            label: di.label,
            value: di.value,
            sort_order: di.sort_order,
            status: di.status,
            remark: di.remark,
            created_at: di.created_at,
            updated_at: di.updated_at,
        }
    }
}

/// 字典公共查询响应项（前端下拉使用）
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DictPublicItem {
    pub label: String,
    pub value: String,
}

/// 创建字典类型请求
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateDictTypeRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub code: String,
    pub status: Option<String>,
    pub remark: Option<String>,
}

/// 更新字典类型请求
#[derive(Debug, Deserialize)]
pub struct UpdateDictTypeRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub remark: Option<String>,
}

/// 创建字典项请求
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateDictItemRequest {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    #[validate(length(min = 1, max = 200))]
    pub value: String,
    pub sort_order: Option<i32>,
    pub status: Option<String>,
    pub remark: Option<String>,
}

/// 更新字典项请求
#[derive(Debug, Deserialize)]
pub struct UpdateDictItemRequest {
    pub label: Option<String>,
    pub value: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<String>,
    pub remark: Option<String>,
}

/// 字典类型列表查询参数
#[derive(Debug, Deserialize)]
pub struct DictTypeListParams {
    pub search: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub per_page: i64,
}
