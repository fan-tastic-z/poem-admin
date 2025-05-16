use nutype::nutype;

use super::{menu::MenuName, page_utils::PageFilter};

// TODO: 信任数据库中获取的数据，并且不需要做数据字段上的new type, 只做字段的取舍
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_by: i64,
    pub created_by_name: String,
    pub is_deletable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateRoleRequest {
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub is_deletable: bool,
    pub menus: Vec<CreateRoleMenuRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateRoleMenuRequest {
    pub menu_id: i64,
    pub menu_name: MenuName,
}

impl CreateRoleMenuRequest {
    pub fn new(menu_id: i64, menu_name: MenuName) -> Self {
        Self { menu_id, menu_name }
    }
}

impl CreateRoleRequest {
    pub fn new(
        name: RoleName,
        description: Option<RoleDescription>,
        is_deletable: bool,
        menus: Vec<CreateRoleMenuRequest>,
    ) -> Self {
        Self {
            name,
            description,
            is_deletable,
            menus,
        }
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_min = 3, len_char_max = 10),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct RoleName(String);

#[nutype(
    sanitize(trim, lowercase),
    validate(len_char_min = 3, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct RoleDescription(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListRoleRequest {
    pub name: Option<RoleName>,
    pub page_filter: PageFilter,
}

impl ListRoleRequest {
    pub fn new(name: Option<RoleName>, page_filter: PageFilter) -> Self {
        Self { name, page_filter }
    }

    pub fn name(&self) -> Option<&RoleName> {
        self.name.as_ref()
    }

    pub fn page_filter(&self) -> &PageFilter {
        &self.page_filter
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListRoleResponseData {
    pub total: i64,
    pub data: Vec<Role>,
}

impl ListRoleResponseData {
    pub fn new(total: i64, data: Vec<Role>) -> Self {
        Self { total, data }
    }
}
