use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[allow(clippy::module_name_repetitions)]
pub enum ApiError {
    RedisError(redis::RedisError),
}

impl From<redis::RedisError> for ApiError {
    fn from(error: redis::RedisError) -> Self {
        Self::RedisError(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::RedisError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error has occured",
            )
                .into_response(),
        }
    }
}
