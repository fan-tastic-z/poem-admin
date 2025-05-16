use std::marker::PhantomData;

use poem::{
    Endpoint, Error, Middleware, Request, Result,
    http::StatusCode,
    web::headers::{self, HeaderMapExt, authorization::Bearer},
};

use crate::{
    cli::Ctx,
    domain::{models::extension_data::ExtensionData, ports::SysService},
};

pub struct AuthMiddleware<S> {
    _phantom: PhantomData<S>,
}

impl<S: SysService> Default for AuthMiddleware<S> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: Endpoint, S: SysService> Middleware<E> for AuthMiddleware<S> {
    type Output = AuthEndpoint<E, S>;

    fn transform(&self, ep: E) -> Self::Output {
        AuthEndpoint {
            inner: ep,
            _phantom: PhantomData,
        }
    }
}

pub struct AuthEndpoint<E, S> {
    inner: E,
    _phantom: PhantomData<S>,
}

impl<E: Endpoint, S: SysService> Endpoint for AuthEndpoint<E, S> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        if let Some(v) = req.headers().typed_get::<headers::Authorization<Bearer>>() {
            let token = v.token();
            let state = req
                .data::<Ctx<S>>()
                .ok_or_else(|| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
            let claims = state.jwt.validate(token).map_err(|_| {
                log::error!("invalid token");
                Error::from_status(StatusCode::UNAUTHORIZED)
            })?;
            let user_id = claims.claims.user_id;
            req.set_data(ExtensionData { user_id });
            self.inner.call(req).await
        } else {
            log::error!("no token");
            Err(Error::from_status(StatusCode::UNAUTHORIZED))
        }
    }
}
