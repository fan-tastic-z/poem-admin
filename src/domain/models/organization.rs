use nutype::nutype;

pub enum OrganizationLimitType {
    Root,                       // 跟组织
    FirstLevel,                 // 一级组织
    SubOrganization,            // 子组织
    SubOrganizationIncludeSelf, // 子组织包含自己
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::FromRow)]
pub struct Organization {
    pub id: i64,
    pub name: String,
    pub parent_id: i64,
    pub parent_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateOrganizationRequest {
    pub name: OrganizationName,
    pub parent_id: i64,
    pub parent_name: Option<OrganizationName>,
}

impl CreateOrganizationRequest {
    pub fn new(
        name: OrganizationName,
        parent_id: i64,
        parent_name: Option<OrganizationName>,
    ) -> Self {
        Self {
            name,
            parent_id,
            parent_name,
        }
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_min = 3, len_char_max = 20),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct OrganizationName(String);
