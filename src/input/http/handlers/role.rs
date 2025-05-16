use poem::{
    Result, handler,
    http::StatusCode,
    web::{Data, Json, Path, Query},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            extension_data::ExtensionData,
            menu::{MenuName, MenuNameError, MenuTree},
            page_utils::{PageFilter, PageNo, PageNoError, PageSize, PageSizeError},
            role::{
                CreateRoleMenuRequest, CreateRoleRequest, GetRoleRequest, GetRoleResponseData,
                ListRoleRequest, Role, RoleDescription, RoleDescriptionError, RoleName,
                RoleNameError,
            },
        },
        ports::SysService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateRoleHttpRequestBody {
    pub name: String,
    pub description: Option<String>,
    pub is_deleteable: bool,
    pub menus: Vec<CreateRoleMenu>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateRoleMenu {
    pub menu_id: i64,
    pub menu_name: String,
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
    MenuName(#[from] MenuNameError),
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
            ParseCreateRoleHttpRequestError::MenuName(e) => {
                format!("Menu name is invalid: {}", e.to_string())
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
        let is_deleteable = self.is_deleteable;
        let menus = self
            .menus
            .into_iter()
            .map(|m| {
                Ok::<_, ParseCreateRoleHttpRequestError>(CreateRoleMenuRequest::new(
                    m.menu_id,
                    MenuName::try_new(m.menu_name)?,
                ))
            })
            .collect::<Result<Vec<_>, ParseCreateRoleHttpRequestError>>()?;
        Ok(CreateRoleRequest::new(
            name,
            description,
            is_deleteable,
            menus,
        ))
    }
}

#[handler]
pub async fn create_role<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
    Json(body): Json<CreateRoleHttpRequestBody>,
) -> Result<ApiSuccess<CreateRoleResponseData>, ApiError> {
    let request = body.try_into_domain()?;
    state
        .sys_service
        .create_role(&request, extension_data.user_id)
        .await
        .map_err(ApiError::from)
        .map(|id| ApiSuccess::new(StatusCode::CREATED, CreateRoleResponseData { id }))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListRoleHttpRequestBody {
    pub name: Option<String>,
    pub page_no: i32,
    pub page_size: i32,
}

impl ListRoleHttpRequestBody {
    fn try_into_domain(self) -> Result<ListRoleRequest, ParseListRoleHttpRequestError> {
        let name = self.name.map(|n| RoleName::try_new(n)).transpose()?;
        let page_no = PageNo::try_new(self.page_no)?;
        let page_size = PageSize::try_new(self.page_size)?;
        let page_filter = PageFilter::new(page_no, page_size);
        Ok(ListRoleRequest::new(name, page_filter))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateAuthorHttpRequestBody {
    pub total: i64,
    pub data: Vec<ListRoleHttpResponseData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListRoleHttpResponseData {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_by: i64,
    pub created_by_name: String,
    pub is_deleteable: bool,
}

impl From<Role> for ListRoleHttpResponseData {
    fn from(role: Role) -> Self {
        Self {
            id: role.id,
            name: role.name,
            description: Some(role.description),
            created_by: role.created_by,
            created_by_name: role.created_by_name,
            is_deleteable: role.is_deletable,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseListRoleHttpRequestError {
    #[error(transparent)]
    RoleName(#[from] RoleNameError),
    #[error(transparent)]
    PageNo(#[from] PageNoError),
    #[error(transparent)]
    PageSize(#[from] PageSizeError),
}

impl From<ParseListRoleHttpRequestError> for ApiError {
    fn from(e: ParseListRoleHttpRequestError) -> Self {
        let message = match e {
            ParseListRoleHttpRequestError::RoleName(e) => {
                format!("Role name is invalid: {}", e.to_string())
            }
            ParseListRoleHttpRequestError::PageNo(e) => {
                format!("Page no is invalid: {}", e.to_string())
            }
            ParseListRoleHttpRequestError::PageSize(e) => {
                format!("Page size is invalid: {}", e.to_string())
            }
        };
        Self::UnprocessableEntity(message)
    }
}

#[handler]
pub async fn list_role<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Query(body): Query<ListRoleHttpRequestBody>,
) -> Result<ApiSuccess<CreateAuthorHttpRequestBody>, ApiError> {
    let req = body.try_into_domain()?;

    state
        .sys_service
        .list_role(req.name.as_ref(), &req.page_filter)
        .await
        .map_err(ApiError::from)
        .map(|roles| {
            let data: Vec<ListRoleHttpResponseData> = roles
                .data
                .into_iter()
                .map(ListRoleHttpResponseData::from)
                .collect();
            ApiSuccess::new(
                StatusCode::OK,
                CreateAuthorHttpRequestBody {
                    total: roles.total,
                    data,
                },
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetRoleHttpResponseData {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub menus: Vec<MenuTree>,
}

impl From<GetRoleResponseData> for GetRoleHttpResponseData {
    fn from(data: GetRoleResponseData) -> Self {
        Self {
            id: data.role.id,
            name: data.role.name,
            description: Some(data.role.description),
            menus: data.menus,
        }
    }
}

#[handler]
pub async fn get_role<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    Path(id): Path<i64>,
) -> Result<ApiSuccess<GetRoleHttpResponseData>, ApiError> {
    let req = GetRoleRequest::new(id);
    state
        .sys_service
        .get_role(&req)
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.into()))
}
