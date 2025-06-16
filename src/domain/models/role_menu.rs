use modql::field::Fields;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct RoleMenu {
    pub role_id: i64,
    pub role_name: String,
    pub menu_id: i64,
    pub menu_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct SaveRoleMenuRequest {
    pub role_id: i64,
    pub role_name: String,
    pub menu_id: i64,
    pub menu_name: String,
}
