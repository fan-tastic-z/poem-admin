use chrono::{DateTime, Utc};
use nutype::nutype;

use super::menu::MenuName;

// TODO: 信任数据库中获取的数据，并且不需要做数据字段上的new type, 只做字段的取舍
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_by: i64,
    pub created_by_name: String,
    pub is_deleteable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateRoleRequest {
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub created_by: i64,
    pub created_by_name: CreateByName,
    pub is_deleteable: bool,
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

    pub fn menu_id(&self) -> i64 {
        self.menu_id
    }

    pub fn menu_name(&self) -> &MenuName {
        &self.menu_name
    }
}

impl CreateRoleRequest {
    pub fn new(
        name: RoleName,
        description: Option<RoleDescription>,
        created_by: i64,
        created_by_name: CreateByName,
        is_deleteable: bool,
        menus: Vec<CreateRoleMenuRequest>,
    ) -> Self {
        Self {
            name,
            description,
            created_by,
            created_by_name,
            is_deleteable,
            menus,
        }
    }

    pub fn name(&self) -> &RoleName {
        &self.name
    }

    pub fn description(&self) -> Option<&RoleDescription> {
        self.description.as_ref()
    }

    pub fn created_by(&self) -> i64 {
        self.created_by
    }

    pub fn created_by_name(&self) -> &CreateByName {
        &self.created_by_name
    }

    pub fn is_deleteable(&self) -> bool {
        self.is_deleteable
    }

    pub fn menus(&self) -> &Vec<CreateRoleMenuRequest> {
        &self.menus
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

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_min = 3, len_char_max = 10),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct CreateByName(String);
