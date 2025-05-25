use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json, Path},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            extension_data::ExtensionData,
            organization::{
                CreateOrganizationRequest, GetOrganizationRequest, GetOrganizationResponseData,
                OrganizationLimitType, OrganizationName, OrganizationNameError, OrganizationTree,
            },
        },
        ports::SysService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateOrganizationHttpRequestBody {
    pub name: String,
    pub parent_id: i64,
    pub parent_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreaterganizationResponseData {
    pub id: i64,
}

#[derive(Debug, Clone, Error)]
pub enum ParseCreateOrganizationHttpRequestBodyError {
    #[error(transparent)]
    Name(#[from] OrganizationNameError),
}

impl From<ParseCreateOrganizationHttpRequestBodyError> for ApiError {
    fn from(e: ParseCreateOrganizationHttpRequestBodyError) -> Self {
        let message = match e {
            ParseCreateOrganizationHttpRequestBodyError::Name(e) => {
                format!("Name is invalid: {}", e)
            }
        };
        Self::UnprocessableEntity(message)
    }
}

impl CreateOrganizationHttpRequestBody {
    pub fn try_into_domain(
        self,
    ) -> Result<CreateOrganizationRequest, ParseCreateOrganizationHttpRequestBodyError> {
        let name = OrganizationName::try_new(self.name)?;
        let parent_id = self.parent_id;
        let parent_name = self
            .parent_name
            .map(OrganizationName::try_new)
            .transpose()?;
        Ok(CreateOrganizationRequest::new(name, parent_id, parent_name))
    }
}

#[handler]
pub async fn create_organization<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Json(body): Json<CreateOrganizationHttpRequestBody>,
) -> Result<ApiSuccess<CreaterganizationResponseData>, ApiError> {
    let request = body.try_into_domain()?;

    state
        .sys_service
        .create_organization(&request)
        .await
        .map_err(ApiError::from)
        .map(|id| ApiSuccess::new(StatusCode::CREATED, CreaterganizationResponseData { id }))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OrganizationTreeHttpRequestBody {
    pub limit_type: OrganizationLimitType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OrganizationTreeResponseData {
    pub organizations: Vec<OrganizationTree>,
}

#[handler]
pub async fn organization_tree<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
    Json(body): Json<OrganizationTreeHttpRequestBody>,
) -> Result<ApiSuccess<OrganizationTreeResponseData>, ApiError> {
    state
        .sys_service
        .organization_tree(extension_data.user_id, body.limit_type)
        .await
        .map_err(ApiError::from)
        .map(|organizations| {
            ApiSuccess::new(
                StatusCode::OK,
                OrganizationTreeResponseData { organizations },
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GetOrganizationHttpRequestBody {
    pub id: i64,
}

impl GetOrganizationHttpRequestBody {
    pub fn into_domain(self, current_user_id: i64) -> GetOrganizationRequest {
        GetOrganizationRequest::new(self.id, current_user_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetOrganizationHttpResponseData {
    pub id: i64,
    pub name: String,
}

impl From<GetOrganizationResponseData> for GetOrganizationHttpResponseData {
    fn from(data: GetOrganizationResponseData) -> Self {
        Self {
            id: data.organization.id,
            name: data.organization.name,
        }
    }
}

#[handler]
pub async fn get_organization<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
    Path(body): Path<GetOrganizationHttpRequestBody>,
) -> Result<ApiSuccess<GetOrganizationHttpResponseData>, ApiError> {
    let req = body.into_domain(extension_data.user_id);
    state
        .sys_service
        .get_organization(&req)
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.into()))
}
