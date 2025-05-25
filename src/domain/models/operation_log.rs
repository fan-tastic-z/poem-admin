use super::account::AccountName;
use crate::utils::ip_validator;
use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::FromRow)]
pub struct OperationLog {
    pub id: i64,
    pub account_id: i64,
    pub account_name: String,
    pub ip_address: String,
    pub user_agent: String,
    pub operation_type: OperationType,
    pub operation_module: String,
    pub operation_description: String,
    pub operation_result: OperationResult,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum OperationResult {
    #[sqlx(rename = "SUCCESS")]
    Success,
    #[sqlx(rename = "FAILED")]
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum OperationType {
    #[sqlx(rename = "CREATE")]
    Create,
    #[sqlx(rename = "UPDATE")]
    Update,
    #[sqlx(rename = "DELETE")]
    Delete,
    #[sqlx(rename = "LOGIN")]
    Login,
    #[sqlx(rename = "LOGOUT")]
    Logout,
    #[sqlx(rename = "VIEW")]
    View,
    #[sqlx(rename = "EXPORT")]
    Export,
    #[sqlx(rename = "IMPORT")]
    Import,
    #[sqlx(rename = "OTHER")]
    Other,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Create => write!(f, "CREATE"),
            OperationType::Update => write!(f, "UPDATE"),
            OperationType::Delete => write!(f, "DELETE"),
            OperationType::Login => write!(f, "LOGIN"),
            OperationType::Logout => write!(f, "LOGOUT"),
            OperationType::View => write!(f, "VIEW"),
            OperationType::Export => write!(f, "EXPORT"),
            OperationType::Import => write!(f, "IMPORT"),
            OperationType::Other => write!(f, "OTHER"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateOperationLogRequest {
    pub account_id: i64,
    pub account_name: AccountName,
    pub ip_address: OperationLogIpAddress,
    pub user_agent: OperationLogUserAgent,
    pub operation_type: OperationType,
    pub operation_module: OperationLogModule,
    pub operation_description: OperationLogDescription,
    pub operation_result: OperationResult,
}

impl CreateOperationLogRequest {
    /// 创建一个新的构建器
    pub fn builder() -> CreateOperationLogRequestBuilder {
        CreateOperationLogRequestBuilder::default()
    }
}

#[derive(Default)]
pub struct CreateOperationLogRequestBuilder {
    account_id: Option<i64>,
    account_name: Option<AccountName>,
    ip_address: Option<OperationLogIpAddress>,
    user_agent: Option<OperationLogUserAgent>,
    operation_type: Option<OperationType>,
    operation_module: Option<OperationLogModule>,
    operation_description: Option<OperationLogDescription>,
    operation_result: Option<OperationResult>,
}

impl CreateOperationLogRequestBuilder {
    pub fn account_id(mut self, account_id: i64) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn account_name(mut self, account_name: AccountName) -> Self {
        self.account_name = Some(account_name);
        self
    }

    pub fn ip_address(mut self, ip_address: OperationLogIpAddress) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn user_agent(mut self, user_agent: OperationLogUserAgent) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn operation_type(mut self, operation_type: OperationType) -> Self {
        self.operation_type = Some(operation_type);
        self
    }

    pub fn operation_module(mut self, operation_module: OperationLogModule) -> Self {
        self.operation_module = Some(operation_module);
        self
    }

    pub fn operation_description(mut self, operation_description: OperationLogDescription) -> Self {
        self.operation_description = Some(operation_description);
        self
    }

    pub fn operation_result(mut self, operation_result: OperationResult) -> Self {
        self.operation_result = Some(operation_result);
        self
    }

    /// 构建 CreateOperationLogRequest
    pub fn build(self) -> Result<CreateOperationLogRequest, &'static str> {
        Ok(CreateOperationLogRequest {
            account_id: self.account_id.ok_or("account_id is required")?,
            account_name: self.account_name.ok_or("account_name is required")?,
            ip_address: self.ip_address.unwrap_or_default(),
            user_agent: self.user_agent.unwrap_or_default(),
            operation_type: self.operation_type.ok_or("operation_type is required")?,
            operation_module: self.operation_module.unwrap_or_default(),
            operation_description: self.operation_description.unwrap_or_default(),
            operation_result: self
                .operation_result
                .ok_or("operation_result is required")?,
        })
    }
}

#[nutype(
    sanitize(trim),
    validate(predicate = |ip| ip_validator::is_valid_ip(ip)),
    default = "0.0.0.0",
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize, Default
    )
)]
pub struct OperationLogIpAddress(String);

#[nutype(
    sanitize(trim),
    default = "unknown",
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize, Default
    )
)]
pub struct OperationLogUserAgent(String);

#[nutype(
    sanitize(trim),
    default = "system",
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize, Default
    )
)]
pub struct OperationLogModule(String);

#[nutype(
    sanitize(trim),
    default = "unknown",
    derive(
        Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Deref, Borrow, TryFrom,
        Serialize, Default
    )
)]
pub struct OperationLogDescription(String);
