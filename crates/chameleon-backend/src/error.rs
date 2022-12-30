use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::jsonapi::{self, Errors, ResourcesDocument};

#[allow(clippy::module_name_repetitions)]
pub enum ApiError {
    JsonApi(Box<jsonapi::Error>),
    Sqlx(sqlx::Error),
}

impl From<jsonapi::Error> for ApiError {
    fn from(error: jsonapi::Error) -> Self {
        Self::JsonApi(error.into())
    }
}

impl From<Box<jsonapi::Error>> for ApiError {
    fn from(error: Box<jsonapi::Error>) -> Self {
        Self::JsonApi(error)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::JsonApi(error) => (
                StatusCode::from_u16(error.status).unwrap(),
                Json(ResourcesDocument::<()> {
                    data: None,
                    errors: Some(Errors(vec![*error])),
                    links: None,
                }),
            )
                .into_response(),
            ApiError::Sqlx(error) => {
                tracing::error!(error =? error, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ResourcesDocument::<()>::internal_server_error()),
                )
                    .into_response()
            }
        }
    }
}
