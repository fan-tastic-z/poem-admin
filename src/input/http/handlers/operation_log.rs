use chrono::{DateTime, Utc};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Query},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli::Ctx,
    domain::{
        models::{
            extension_data::ExtensionData,
            operation_log::{ListOperationLogRequest, ListOperationLogResponseData},
            page_utils::{PageFilter, PageNo, PageNoError, PageSize, PageSizeError},
        },
        ports::SysService,
    },
    input::http::response::{ApiError, ApiSuccess},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListOperationLogHttpRequestBody {
    pub page_no: i32,
    pub page_size: i32,
}

impl ListOperationLogHttpRequestBody {
    pub fn try_into_domain(
        self,
        current_user_id: i64,
    ) -> Result<ListOperationLogRequest, ParseListOperationLogHttpRequestBodyError> {
        let page_no = PageNo::try_new(self.page_no)?;
        let page_size = PageSize::try_new(self.page_size)?;
        let page_filter = PageFilter::new(page_no, page_size);
        Ok(ListOperationLogRequest {
            page_filter,
            current_user_id,
        })
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseListOperationLogHttpRequestBodyError {
    #[error(transparent)]
    PageNo(#[from] PageNoError),
    #[error(transparent)]
    PageSize(#[from] PageSizeError),
}

impl From<ParseListOperationLogHttpRequestBodyError> for ApiError {
    fn from(e: ParseListOperationLogHttpRequestBodyError) -> Self {
        let message = match e {
            ParseListOperationLogHttpRequestBodyError::PageNo(e) => {
                format!("Page no is invalid: {}", e)
            }
            ParseListOperationLogHttpRequestBodyError::PageSize(e) => {
                format!("Page size is invalid: {}", e)
            }
        };
        ApiError::UnprocessableEntity(message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ListOperationLogHttpResponseData {
    pub total: i64,
    pub data: Vec<OperationLogData>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct OperationLogData {
    pub id: i64,
    pub account_id: i64,
    pub account_name: String,
    pub ip_address: String,
    pub user_agent: String,
    pub operation_type: String,
    pub operation_module: String,
    pub operation_description: String,
    pub operation_result: String,
    pub created_at: DateTime<Utc>,
}

impl From<ListOperationLogResponseData> for ListOperationLogHttpResponseData {
    fn from(data: ListOperationLogResponseData) -> Self {
        Self {
            total: data.total,
            data: data
                .operation_logs
                .into_iter()
                .map(|operation_log| OperationLogData {
                    id: operation_log.id,
                    account_id: operation_log.account_id,
                    account_name: operation_log.account_name,
                    ip_address: operation_log.ip_address,
                    user_agent: operation_log.user_agent,
                    operation_type: operation_log.operation_type.to_string(),
                    operation_module: operation_log.operation_module,
                    operation_description: operation_log.operation_description,
                    operation_result: operation_log.operation_result.to_string(),
                    created_at: operation_log.created_at,
                })
                .collect(),
        }
    }
}

#[handler]
pub async fn list_operation_log<S: SysService + Send + Sync + 'static>(
    state: Data<&Ctx<S>>,
    extension_data: Data<&ExtensionData>,
    Query(body): Query<ListOperationLogHttpRequestBody>,
) -> Result<ApiSuccess<ListOperationLogHttpResponseData>, ApiError> {
    let req = body.try_into_domain(extension_data.user_id)?;
    state
        .sys_service
        .list_operation_log(&req)
        .await
        .map_err(ApiError::from)
        .map(|data| ApiSuccess::new(StatusCode::OK, data.into()))
}
