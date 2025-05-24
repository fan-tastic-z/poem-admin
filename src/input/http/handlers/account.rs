use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Query},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            account::{
                AccountEmail, AccountEmailError, AccountName, AccountNameError, AccountPassword,
                AccountPasswordError, AcountData, CreateAccountRequest, CurrentAccountResponseData,
                ListAccountRequest, ListAccountResponseData,
            },
            extension_data::ExtensionData,
            menu::MenuTree,
            organization::{OrganizationName, OrganizationNameError},
            page_utils::{PageFilter, PageNo, PageNoError, PageSize, PageSizeError},
            role::{RoleName, RoleNameError},
        },
        ports::SysService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateAccountHttpRequestBody {
    pub name: String,
    pub password: String,
    pub email: Option<String>,
    pub organization_id: i64,
    pub organization_name: String,
    pub role_id: i64,
    pub role_name: String,
}

impl CreateAccountHttpRequestBody {
    pub fn try_into_domain(
        self,
    ) -> Result<CreateAccountRequest, ParseCreateAccountHttpRequestBodyError> {
        let name = AccountName::try_new(self.name)?;
        let password = AccountPassword::try_new(self.password)?;

        let email = self.email.map(AccountEmail::try_new).transpose()?;
        let organization_name = OrganizationName::try_new(self.organization_name)?;
        let role_name = RoleName::try_new(self.role_name)?;
        Ok(CreateAccountRequest::new(
            name,
            password,
            email,
            self.organization_id,
            organization_name,
            self.role_id,
            role_name,
            true,
        ))
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseCreateAccountHttpRequestBodyError {
    #[error(transparent)]
    Name(#[from] AccountNameError),
    #[error(transparent)]
    Password(#[from] AccountPasswordError),
    #[error(transparent)]
    Email(#[from] AccountEmailError),
    #[error(transparent)]
    OrganizationName(#[from] OrganizationNameError),
    #[error(transparent)]
    RoleName(#[from] RoleNameError),
}

impl From<ParseCreateAccountHttpRequestBodyError> for ApiError {
    fn from(e: ParseCreateAccountHttpRequestBodyError) -> Self {
        let message = match e {
            ParseCreateAccountHttpRequestBodyError::Name(e) => {
                format!("Name is invalid: {}", e.to_string())
            }
            ParseCreateAccountHttpRequestBodyError::Password(e) => {
                format!("Password is invalid: {}", e.to_string())
            }
            ParseCreateAccountHttpRequestBodyError::Email(e) => {
                format!("Email is invalid: {}", e.to_string())
            }
            ParseCreateAccountHttpRequestBodyError::OrganizationName(e) => {
                format!("Organization name is invalid: {}", e.to_string())
            }
            ParseCreateAccountHttpRequestBodyError::RoleName(e) => {
                format!("Role name is invalid: {}", e.to_string())
            }
        };
        ApiError::UnprocessableEntity(message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateAccountHttpResponseData {
    pub id: i64,
}

#[handler]
pub async fn create_account<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
    Json(body): Json<CreateAccountHttpRequestBody>,
) -> Result<ApiSuccess<CreateAccountHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    state
        .sys_service
        .create_account(&req, extension_data.user_id)
        .await
        .map_err(ApiError::from)
        .map(|id| ApiSuccess::new(StatusCode::CREATED, CreateAccountHttpResponseData { id }))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurrentAccountHttpResponseData {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub organization_id: i64,
    pub organization_name: String,
    pub role_id: i64,
    pub role_name: String,
    pub menus: Vec<MenuTree>,
}

impl From<CurrentAccountResponseData> for CurrentAccountHttpResponseData {
    fn from(data: CurrentAccountResponseData) -> Self {
        Self {
            id: data.account.id,
            name: data.account.name,
            email: data.account.email,
            organization_id: data.account.organization_id,
            organization_name: data.account.organization_name,
            role_id: data.account.role_id,
            role_name: data.account.role_name,
            menus: data.menus,
        }
    }
}

#[handler]
pub async fn current_account<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
) -> Result<ApiSuccess<CurrentAccountHttpResponseData>, ApiError> {
    state
        .sys_service
        .current_account(extension_data.user_id)
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.into()))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListAccountHttpRequestBody {
    pub account_name: Option<String>,
    pub page_no: i32,
    pub page_size: i32,
    pub organization_id: Option<i64>,
}

impl ListAccountHttpRequestBody {
    pub fn try_into_domain(
        self,
        current_user_id: i64,
    ) -> Result<ListAccountRequest, ParseListAccountHttpRequestBodyError> {
        let account_name = self
            .account_name
            .map(|n| AccountName::try_new(n))
            .transpose()?;
        let page_no = PageNo::try_new(self.page_no)?;
        let page_size = PageSize::try_new(self.page_size)?;
        let page_filter = PageFilter::new(page_no, page_size);
        Ok(ListAccountRequest::new(
            account_name,
            page_filter,
            current_user_id,
            self.organization_id,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListAccountHttpResponseData {
    pub total: i64,
    pub data: Vec<AcountData>,
}

impl From<ListAccountResponseData> for ListAccountHttpResponseData {
    fn from(data: ListAccountResponseData) -> Self {
        Self {
            total: data.total,
            data: data.data,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseListAccountHttpRequestBodyError {
    #[error(transparent)]
    AccountName(#[from] AccountNameError),
    #[error(transparent)]
    PageNo(#[from] PageNoError),
    #[error(transparent)]
    PageSize(#[from] PageSizeError),
}

impl From<ParseListAccountHttpRequestBodyError> for ApiError {
    fn from(e: ParseListAccountHttpRequestBodyError) -> Self {
        let message = match e {
            ParseListAccountHttpRequestBodyError::AccountName(e) => {
                format!("Account name is invalid: {}", e.to_string())
            }
            ParseListAccountHttpRequestBodyError::PageNo(e) => {
                format!("Page no is invalid: {}", e.to_string())
            }
            ParseListAccountHttpRequestBodyError::PageSize(e) => {
                format!("Page size is invalid: {}", e.to_string())
            }
        };
        ApiError::UnprocessableEntity(message)
    }
}

#[handler]
pub async fn list_account<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
    Query(body): Query<ListAccountHttpRequestBody>,
) -> Result<ApiSuccess<ListAccountHttpResponseData>, ApiError> {
    let req = body.try_into_domain(extension_data.user_id)?;
    state
        .sys_service
        .list_account(&req)
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.into()))
}
