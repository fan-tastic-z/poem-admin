use super::account::{AccountName, AccountPassword};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoginRequest {
    pub username: AccountName,
    pub password: AccountPassword,
}

impl LoginRequest {
    pub fn new(username: AccountName, password: AccountPassword) -> Self {
        Self { username, password }
    }
}
