use error_stack::Report;
use poem::{IntoResponse, Response, error::ResponseError, http::StatusCode, web::Json};
use serde::Serialize;
use std::fmt;

use crate::errors::Error;

#[derive(Debug, Clone)]
pub struct ApiSuccess<T: Serialize + PartialEq + Send + Sync>(StatusCode, Json<ApiResponseBody<T>>);

impl<T> PartialEq for ApiSuccess<T>
where
    T: Serialize + PartialEq + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.0 == other.1.0
    }
}

impl<T: Serialize + PartialEq + Send + Sync> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl<T: Serialize + PartialEq + Send + Sync> ApiSuccess<T> {
    pub fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq + Send + Sync> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq + Send + Sync> ApiResponseBody<T> {
    pub fn new(status_code: StatusCode, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorData {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
    Unauthorized(String),
    BadRequest(String),
}

impl From<Report<Error>> for ApiError {
    fn from(e: Report<Error>) -> Self {
        log::error!("ApiError: {:#?}", e);
        match e.downcast_ref::<Error>() {
            Some(Error::BadRequest(message)) => Self::BadRequest(message.clone()),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

impl ResponseError for ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn as_response(&self) -> Response {
        use ApiError::*;

        match self {
            InternalServerError(_) => Json(ApiResponseBody::new_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ))
            .into_response(),
            Unauthorized(message) => {
                log::error!("Unauthorized: {}", message);
                Json(ApiResponseBody::new_error(
                    StatusCode::UNAUTHORIZED,
                    "Unauthorized".to_string(),
                ))
                .into_response()
            }
            UnprocessableEntity(message) => Json(ApiResponseBody::new_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                message.to_string(),
            ))
            .into_response(),
            BadRequest(message) => Json(ApiResponseBody::new_error(
                StatusCode::BAD_REQUEST,
                message.to_string(),
            ))
            .into_response(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            Self::UnprocessableEntity(msg) => write!(f, "Unprocessable entity: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}
