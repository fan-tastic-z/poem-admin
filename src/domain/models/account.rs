use email_address::EmailAddress;
use modql::field::Fields;
use nutype::nutype;
use sea_query::{Nullable, Value};
use serde::Serialize;

use super::{
    menu::MenuTree, organization::OrganizationName, page_utils::PageFilter, role::RoleName,
};

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
    pub phone: Option<String>,
    pub is_deletable: bool,
    #[sqlx(skip)]
    pub is_authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fields)]
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
    /// Create a new account request with required fields
    pub fn new(
        name: AccountName,
        password: AccountPassword,
        organization_id: i64,
        organization_name: OrganizationName,
        role_id: i64,
        role_name: RoleName,
    ) -> Self {
        Self {
            name,
            password,
            email: None,
            organization_id,
            organization_name,
            role_id,
            role_name,
            is_deletable: true, // 默认可删除
        }
    }
    pub fn with_password(mut self, password: AccountPassword) -> Self {
        self.password = password;
        self
    }

    /// Set email for the account
    pub fn with_email(mut self, email: AccountEmail) -> Self {
        self.email = Some(email);
        self
    }

    /// Set whether the account is deletable
    pub fn with_deletable(mut self, is_deletable: bool) -> Self {
        self.is_deletable = is_deletable;
        self
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_min = 4, len_char_max = 20),
    default = "unknown",
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize, Default
    )
)]
pub struct AccountName(String);

// Implement From<AccountName> for sea_query::Value
impl From<AccountName> for Value {
    fn from(account_name: AccountName) -> Self {
        Value::String(Some(Box::new(account_name.into_inner())))
    }
}

// Implement Nullable for AccountName (needed for Option<AccountName>)
impl Nullable for AccountName {
    fn null() -> Value {
        Value::String(None)
    }
}

#[nutype(
    sanitize(trim, lowercase),
    validate(predicate = |email| email.is_empty() || EmailAddress::is_valid(email)),
    derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom)
)]
pub struct AccountEmail(String);

// Implement From<AccountEmail> for sea_query::Value
impl From<AccountEmail> for Value {
    fn from(account_email: AccountEmail) -> Self {
        Value::String(Some(Box::new(account_email.into_inner())))
    }
}

// Implement Nullable for AccountEmail (needed for Option<AccountEmail>)
impl Nullable for AccountEmail {
    fn null() -> Value {
        Value::String(None)
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_min = 8, len_char_max = 128),
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom
    )
)]
pub struct AccountPassword(String);

// Implement From<AccountPassword> for sea_query::Value
impl From<AccountPassword> for Value {
    fn from(account_password: AccountPassword) -> Self {
        Value::String(Some(Box::new(account_password.into_inner())))
    }
}

// Implement Nullable for AccountPassword (needed for Option<AccountPassword>)
impl Nullable for AccountPassword {
    fn null() -> Value {
        Value::String(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrentAccountResponseData {
    pub account: Account,
    pub menus: Vec<MenuTree>,
}

impl CurrentAccountResponseData {
    pub fn new(account: Account, menus: Vec<MenuTree>) -> Self {
        Self { account, menus }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListAccountRequest {
    pub account_name: Option<AccountName>,
    pub page_filter: PageFilter,
    pub current_user_id: i64,
    pub organization_id: Option<i64>,
}

impl ListAccountRequest {
    pub fn new(
        account_name: Option<AccountName>,
        page_filter: PageFilter,
        current_user_id: i64,
        organization_id: Option<i64>,
    ) -> Self {
        Self {
            account_name,
            page_filter,
            current_user_id,
            organization_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListAccountResponseData {
    pub total: i64,
    pub data: Vec<Account>,
}

impl ListAccountResponseData {
    pub fn new(total: i64, data: Vec<Account>) -> Self {
        Self { total, data }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetAccountRequest {
    pub id: i64,
    pub current_user_id: i64,
}

impl GetAccountRequest {
    pub fn new(id: i64, current_user_id: i64) -> Self {
        Self {
            id,
            current_user_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetAccountResponseData {
    pub account: Account,
    pub menus: Vec<MenuTree>,
}

impl GetAccountResponseData {
    pub fn new(account: Account, menus: Vec<MenuTree>) -> Self {
        Self { account, menus }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_email_empty_is_valid() {
        // 空字符串应该是有效的
        let result = AccountEmail::try_new("".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_account_email_whitespace_becomes_empty() {
        // 只有空白字符的字符串经过 trim 后变成空字符串，应该是有效的
        let result = AccountEmail::try_new("   ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "");
    }

    #[test]
    fn test_account_email_valid_email() {
        // 有效的邮箱地址应该通过验证
        let result = AccountEmail::try_new("test@example.com".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "test@example.com");
    }

    #[test]
    fn test_account_email_invalid_email() {
        // 无效的邮箱地址应该失败
        let result = AccountEmail::try_new("invalid-email".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_account_email_sanitization() {
        // 测试清理功能：trim 和 lowercase
        let result = AccountEmail::try_new("  Test@Example.COM  ".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "test@example.com");
    }
}
