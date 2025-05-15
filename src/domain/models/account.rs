use email_address::EmailAddress;
use nutype::nutype;
use serde::Serialize;
use thiserror::Error;

use super::{organization::OrganizationName, role::RoleName};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub email: Option<String>,
    pub organization_id: i64,
    pub organization_name: String,
    pub role_id: i64,
    pub role_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateAccountRequest {
    pub name: AccountName,
    pub password: AccountPassword,
    pub email: Option<AccountEmail>,
    pub organization_id: i64,
    pub organization_name: OrganizationName,
    pub role_id: i64,
    pub role_name: RoleName,
    pub is_deletable: bool,
}

impl CreateAccountRequest {
    pub fn new(
        name: AccountName,
        password: AccountPassword,
        email: Option<AccountEmail>,
        organization_id: i64,
        organization_name: OrganizationName,
        role_id: i64,
        role_name: RoleName,
        is_deletable: bool,
    ) -> Self {
        Self {
            name,
            password,
            email,
            organization_id,
            organization_name,
            role_id,
            role_name,
            is_deletable,
        }
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_min = 4, len_char_max = 10),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize
    )
)]
pub struct AccountName(String);

#[nutype(
    sanitize(trim, lowercase),
    validate(with=valid_user_email, error=AccountEmailError),
    derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom)
)]
pub struct AccountEmail(String);

#[derive(Debug, Error, Clone)]
#[error("invalid email:{0}")]
pub struct AccountEmailError(String);

fn valid_user_email(email: &str) -> Result<(), AccountEmailError> {
    let res = EmailAddress::is_valid(email);
    if res {
        return Ok(());
    }
    return Err(AccountEmailError(email.to_string()));
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_min = 8, len_char_max = 128),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom
    )
)]
pub struct AccountPassword(String);
