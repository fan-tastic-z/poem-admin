use modql::field::Fields;
use nutype::nutype;
use sea_query::{Nullable, Value};

use crate::domain::models::account::AccountName;

use super::{
    menu::{MenuName, MenuTree},
    page_utils::PageFilter,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_by: i64,
    pub created_by_name: String,
    pub is_deletable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct SaveRoleRequest {
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub is_deletable: bool,
    pub created_by: i64,
    pub created_by_name: AccountName,
}

impl SaveRoleRequest {
    pub fn new(
        name: RoleName,
        description: Option<RoleDescription>,
        is_deletable: bool,
        created_by: i64,
        created_by_name: AccountName,
    ) -> Self {
        Self {
            name,
            description,
            is_deletable,
            created_by,
            created_by_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct CreateRoleRequest {
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub is_deletable: bool,
    pub menus: RoleMenuList,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
pub struct CreateRoleMenuRequest {
    pub menu_id: i64,
    pub menu_name: MenuName,
}

impl CreateRoleMenuRequest {
    pub fn new(menu_id: i64, menu_name: MenuName) -> Self {
        Self { menu_id, menu_name }
    }
}

impl From<CreateRoleMenuRequest> for Value {
    fn from(menu: CreateRoleMenuRequest) -> Self {
        Value::String(Some(Box::new(format!(
            "{}:{}",
            menu.menu_id, menu.menu_name
        ))))
    }
}

// Create a wrapper type to implement From trait
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleMenuList(pub Vec<CreateRoleMenuRequest>);

impl From<Vec<CreateRoleMenuRequest>> for RoleMenuList {
    fn from(menus: Vec<CreateRoleMenuRequest>) -> Self {
        Self(menus)
    }
}

impl AsRef<[CreateRoleMenuRequest]> for RoleMenuList {
    fn as_ref(&self) -> &[CreateRoleMenuRequest] {
        &self.0
    }
}

impl std::ops::Deref for RoleMenuList {
    type Target = Vec<CreateRoleMenuRequest>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<RoleMenuList> for Value {
    fn from(menus: RoleMenuList) -> Self {
        let menu_values: Vec<Value> = menus.0.into_iter().map(Value::from).collect();
        Value::Array(sea_query::ArrayType::String, Some(Box::new(menu_values)))
    }
}

impl CreateRoleRequest {
    pub fn new(
        name: RoleName,
        description: Option<RoleDescription>,
        is_deletable: bool,
        menus: impl Into<RoleMenuList>,
    ) -> Self {
        Self {
            name,
            description,
            is_deletable,
            menus: menus.into(),
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

impl From<RoleName> for Value {
    fn from(role_name: RoleName) -> Self {
        Value::String(Some(Box::new(role_name.into_inner())))
    }
}

impl Nullable for RoleName {
    fn null() -> Value {
        Value::String(None)
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(len_char_min = 3, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct RoleDescription(String);

impl From<RoleDescription> for Value {
    fn from(role_description: RoleDescription) -> Self {
        Value::String(Some(Box::new(role_description.into_inner())))
    }
}

impl Nullable for RoleDescription {
    fn null() -> Value {
        Value::String(None)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetRoleRequest {
    pub id: i64,
}

impl GetRoleRequest {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetRoleResponseData {
    pub role: Role,
    pub menus: Vec<MenuTree>,
}

impl GetRoleResponseData {
    pub fn new(role: Role, menus: Vec<MenuTree>) -> Self {
        Self { role, menus }
    }
}
