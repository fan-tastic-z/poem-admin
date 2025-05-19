use nutype::nutype;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct Route {
    pub name: String,
    pub method: String,
    pub menu_id: i64,
    pub menu_name: String,
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct RouteMethod(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct RoutePath(String);
