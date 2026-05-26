use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::menu::{CreateMenuRequest, MenuResponse, MenuTreeNode, UpdateMenuRequest};
use crate::repository::menu_repo;

/// List all menus (admin)
pub async fn list_menus(pool: &PgPool) -> Result<Vec<MenuResponse>, AppError> {
    let menus = menu_repo::list_all(pool).await?;
    Ok(menus.into_iter().map(MenuResponse::from).collect())
}

/// Get menu tree filtered by user permissions
pub async fn get_menu_tree(
    pool: &PgPool,
    permission_names: &[String],
) -> Result<Vec<MenuTreeNode>, AppError> {
    let menus = menu_repo::list_by_permission_names(pool, permission_names).await?;
    let responses: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();
    Ok(build_tree(&responses))
}

/// Build a tree from flat menu list
fn build_tree(menus: &[MenuResponse]) -> Vec<MenuTreeNode> {
    // Find root items (no parent)
    let roots: Vec<&MenuResponse> = menus.iter().filter(|m| m.parent_id.is_none()).collect();
    roots
        .into_iter()
        .map(|root| build_node(root, menus))
        .collect()
}

fn build_node(menu: &MenuResponse, all: &[MenuResponse]) -> MenuTreeNode {
    let children: Vec<MenuTreeNode> = all
        .iter()
        .filter(|m| m.parent_id == Some(menu.id))
        .map(|child| build_node(child, all))
        .collect();

    MenuTreeNode {
        id: menu.id,
        title_key: menu.title_key.clone(),
        title_label: menu.title_label.clone(),
        path: menu.path.clone(),
        icon: menu.icon.clone(),
        sort_order: menu.sort_order,
        children,
    }
}

/// Create a new menu item
pub async fn create_menu(
    pool: &PgPool,
    req: &CreateMenuRequest,
) -> Result<MenuResponse, AppError> {
    // Validate parent exists if specified
    if let Some(parent_id) = req.parent_id {
        if menu_repo::find_by_id(pool, parent_id).await?.is_none() {
            return Err(AppError::BadRequest("Parent menu not found".into()));
        }
    }
    let menu = menu_repo::create(pool, req).await?;
    Ok(MenuResponse::from(menu))
}

/// Update a menu item
pub async fn update_menu(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateMenuRequest,
) -> Result<MenuResponse, AppError> {
    // Check exists
    menu_repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Menu not found".into()))?;

    let menu = menu_repo::update(pool, id, req).await?;
    Ok(MenuResponse::from(menu))
}

/// Delete a menu item
pub async fn delete_menu(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    menu_repo::delete(pool, id).await
}
