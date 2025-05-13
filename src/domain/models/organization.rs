use nutype::nutype;

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

    pub fn name(&self) -> &OrganizationName {
        &self.name
    }

    pub fn parent_id(&self) -> i64 {
        self.parent_id
    }

    pub fn parent_name(&self) -> Option<&OrganizationName> {
        self.parent_name.as_ref()
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
