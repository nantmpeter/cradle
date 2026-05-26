use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 部门数据库模型
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Department {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub sort_order: i32,
    pub status: String,
    pub leader: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// 部门响应 DTO
#[derive(Debug, Serialize, Clone)]
pub struct DepartmentResponse {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub sort_order: i32,
    pub status: String,
    pub leader: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Department> for DepartmentResponse {
    fn from(d: Department) -> Self {
        Self {
            id: d.id,
            parent_id: d.parent_id,
            name: d.name,
            code: d.code,
            sort_order: d.sort_order,
            status: d.status,
            leader: d.leader,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

/// 部门树形节点
#[derive(Debug, Serialize)]
pub struct DepartmentTreeNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub sort_order: i32,
    pub status: String,
    pub leader: Option<String>,
    pub children: Vec<DepartmentTreeNode>,
}

/// 创建部门请求
#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateDepartmentRequest {
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub code: String,
    pub sort_order: Option<i32>,
    pub status: Option<String>,
    pub leader: Option<String>,
}

/// 更新部门请求
#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<String>,
    pub leader: Option<String>,
}
