use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            account::{
                AccountEmail, AccountEmailError, AccountName, AccountNameError, AccountPassword,
                AccountPasswordError, CreateAccountRequest,
            },
            organization::{OrganizationName, OrganizationNameError},
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
    Json(body): Json<CreateAccountHttpRequestBody>,
) -> Result<ApiSuccess<CreateAccountHttpResponseData>, ApiError> {
    let req = body.try_into_domain()?;
    state
        .sys_service
        .create_account(&req)
        .await
        .map_err(ApiError::from)
        .map(|id| ApiSuccess::new(StatusCode::CREATED, CreateAccountHttpResponseData { id }))
}
