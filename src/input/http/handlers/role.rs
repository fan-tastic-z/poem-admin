use poem::{
    Result, handler,
    http::StatusCode,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::role::{
            CreateByName, CreateByNameError, CreateRoleRequest, RoleDescription,
            RoleDescriptionError, RoleName, RoleNameError,
        },
        ports::SysService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateRoleHttpRequestBody {
    pub name: String,
    pub description: Option<String>,
    // TODO: 需要从上下文获取
    pub created_by: i64,
    pub created_by_name: String,
    pub is_deleteable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateRoleResponseData {
    pub id: i64,
}

#[derive(Debug, Clone, Error)]
enum ParseCreateRoleHttpRequestError {
    #[error(transparent)]
    RoleName(#[from] RoleNameError),
    #[error(transparent)]
    RoleDescription(#[from] RoleDescriptionError),
    #[error(transparent)]
    CreateByName(#[from] CreateByNameError),
}

impl From<ParseCreateRoleHttpRequestError> for ApiError {
    fn from(e: ParseCreateRoleHttpRequestError) -> Self {
        let message = match e {
            ParseCreateRoleHttpRequestError::RoleName(e) => {
                format!("Role name is invalid: {}", e.to_string())
            }
            ParseCreateRoleHttpRequestError::RoleDescription(e) => {
                format!("Role description is invalid: {}", e.to_string())
            }
            ParseCreateRoleHttpRequestError::CreateByName(e) => {
                format!("Create by name is invalid: {}", e.to_string())
            }
        };
        Self::UnprocessableEntity(message)
    }
}

impl CreateRoleHttpRequestBody {
    fn try_into_domain(self) -> Result<CreateRoleRequest, ParseCreateRoleHttpRequestError> {
        let name = RoleName::try_new(self.name)?;
        let description = self
            .description
            .map(|d| RoleDescription::try_new(d))
            .transpose()?;
        let created_by = self.created_by;
        let created_by_name = CreateByName::try_new(self.created_by_name)?;
        let is_deleteable = self.is_deleteable;
        Ok(CreateRoleRequest::new(
            name,
            description,
            created_by,
            created_by_name,
            is_deleteable,
        ))
    }
}

#[handler]
pub async fn create_role<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Json(body): Json<CreateRoleHttpRequestBody>,
) -> Result<ApiSuccess<CreateRoleResponseData>, ApiError> {
    let request = body.try_into_domain()?;
    state
        .sys_service
        .create_role(&request)
        .await
        .map_err(ApiError::from)
        .map(|id| ApiSuccess::new(StatusCode::CREATED, CreateRoleResponseData { id }))
}
