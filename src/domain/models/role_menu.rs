use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct RoleMenu {
    pub role_id: i64,
    pub role_name: String,
    pub menu_id: i64,
    pub menu_name: String,
}
