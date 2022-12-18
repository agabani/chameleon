use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[allow(clippy::module_name_repetitions)]
pub enum ApiError {
    SqlxError(sqlx::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::SqlxError(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::SqlxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error has occurred",
            )
                .into_response(),
        }
    }
}
