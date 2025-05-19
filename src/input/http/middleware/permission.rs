use std::marker::PhantomData;

use poem::{Endpoint, Error, Middleware, Request, Result, http::StatusCode};

use crate::{
    cli::Ctx,
    domain::{
        models::{
            extension_data::ExtensionData,
            route::{RouteMethod, RoutePath},
        },
        ports::SysService,
    },
};

pub struct PermissionMiddleware<S> {
    _phantom: PhantomData<S>,
}

impl<S: SysService> Default for PermissionMiddleware<S> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: Endpoint, S: SysService> Middleware<E> for PermissionMiddleware<S> {
    type Output = PermissionEndpoint<E, S>;

    fn transform(&self, ep: E) -> Self::Output {
        PermissionEndpoint {
            inner: ep,
            _phantom: PhantomData,
        }
    }
}

pub struct PermissionEndpoint<E, S> {
    inner: E,
    _phantom: PhantomData<S>,
}

impl<E: Endpoint, S: SysService> Endpoint for PermissionEndpoint<E, S> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let extension_data = req
            .extensions()
            .get::<ExtensionData>()
            .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;
        let path = req.original_uri().path().to_string();
        let method = req.method().to_string();

        let state = req
            .data::<Ctx<S>>()
            .ok_or_else(|| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        let res = state
            .sys_service
            .check_permission(
                extension_data.user_id,
                &RoutePath::try_new(&path).map_err(|e| {
                    log::error!("failed to parse route path: {:?}", e);
                    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                })?,
                &RouteMethod::try_new(&method).map_err(|e| {
                    log::error!("failed to parse route method: {:?}", e);
                    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
                })?,
            )
            .await
            .map_err(|e| {
                log::error!("failed to check permission: {:?}", e);
                Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        if !res {
            return Err(Error::from_status(StatusCode::FORBIDDEN));
        }
        self.inner.call(req).await
    }
}
