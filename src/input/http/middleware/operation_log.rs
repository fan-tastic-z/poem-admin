use std::marker::PhantomData;

use poem::{
    Endpoint, Error, Middleware, Request, Result,
    http::{Method, StatusCode, header::USER_AGENT},
};

use crate::{
    cli::Ctx,
    domain::{
        models::{
            account::{AccountName, GetAccountRequest},
            extension_data::ExtensionData,
            operation_log::{
                CreateOperationLogRequest, OperationLogDescription, OperationLogIpAddress,
                OperationLogModule, OperationLogUserAgent, OperationResult, OperationType,
            },
        },
        ports::SysService,
    },
};

pub struct OperationLogMiddleware<S> {
    _phantom: PhantomData<S>,
}

impl<S: SysService> Default for OperationLogMiddleware<S> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: Endpoint, S: SysService> Middleware<E> for OperationLogMiddleware<S> {
    type Output = OperationLogEndpoint<E, S>;

    fn transform(&self, ep: E) -> Self::Output {
        OperationLogEndpoint {
            inner: ep,
            _phantom: PhantomData,
        }
    }
}

pub struct OperationLogEndpoint<E, S> {
    inner: E,
    _phantom: PhantomData<S>,
}

impl<E: Endpoint, S: SysService> Endpoint for OperationLogEndpoint<E, S> {
    type Output = E::Output;
    async fn call(&self, req: Request) -> Result<Self::Output> {
        let method = req.method().to_string();
        let path = req.original_uri().path().to_string();
        let user_agent = req
            .headers()
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let ip_address = req
            .remote_addr()
            .0
            .as_socket_addr()
            .map(|addr| addr.ip().to_string());
        let extension_data = req.extensions().get::<ExtensionData>().cloned();
        let state = req
            .data::<Ctx<S>>()
            .ok_or_else(|| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
            .clone();
        let result = self.inner.call(req).await;
        if method == Method::GET.to_string() {
            return result;
        }
        if let Some(extension_data) = &extension_data {
            let (status_code, _) = match &result {
                Ok(_) => (200, None),
                Err(err) => {
                    let status = err.status();
                    (status.as_u16() as i32, Some(format!("{:?}", err)))
                }
            };
            let (operation_type, operation_module, operation_description) =
                determine_operation_info(&method, &path);
            let operation_result = if status_code == 200 {
                OperationResult::Success
            } else {
                OperationResult::Failed
            };
            if let Ok(res) = state
                .sys_service
                .get_account(&GetAccountRequest::new(
                    extension_data.user_id,
                    extension_data.user_id,
                ))
                .await
            {
                let operation_log_request = CreateOperationLogRequest::builder()
                    .account_id(res.account.id)
                    .account_name(AccountName::try_from(res.account.name).unwrap_or_default())
                    .ip_address(
                        OperationLogIpAddress::try_from(ip_address.unwrap_or_default())
                            .unwrap_or_default(),
                    )
                    .user_agent(
                        OperationLogUserAgent::try_from(user_agent.unwrap_or_default())
                            .unwrap_or_default(),
                    )
                    .operation_type(operation_type)
                    .operation_module(
                        OperationLogModule::try_from(operation_module).unwrap_or_default(),
                    )
                    .operation_description(
                        OperationLogDescription::try_from(operation_description)
                            .unwrap_or_default(),
                    )
                    .operation_result(operation_result)
                    .build()
                    .expect("Failed to build operation log request");
                tokio::spawn(async move {
                    if let Err(e) = state
                        .sys_service
                        .create_operation_log(&operation_log_request)
                        .await
                    {
                        log::error!("Failed to create operation log: {:?}", e);
                    }
                });
            }
        }
        result
    }
}

fn determine_operation_info(method: &str, path: &str) -> (OperationType, String, String) {
    let operation_type = match method {
        "POST" => {
            if path.contains("/login") {
                OperationType::Login
            } else {
                OperationType::Create
            }
        }
        "PUT" | "PATCH" => OperationType::Update,
        "DELETE" => OperationType::Delete,
        "GET" => OperationType::View,
        _ => OperationType::Other,
    };
    let (module, description) = if path.contains("/accounts") {
        ("account".to_string(), format!("{} account", operation_type))
    } else if path.contains("/roles") {
        ("role".to_string(), format!("{} role", operation_type))
    } else if path.contains("/organizations") {
        (
            "organization".to_string(),
            format!("{} organization", operation_type),
        )
    } else if path.contains("/menus") {
        ("menu".to_string(), format!("{} menu", operation_type))
    } else if path.contains("/login") {
        ("auth".to_string(), "User login".to_string())
    } else {
        (
            "system".to_string(),
            format!("{} operation", operation_type),
        )
    };
    (operation_type, module, description)
}
