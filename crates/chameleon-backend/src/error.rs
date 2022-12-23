use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::jsonapi::{self, Document, Errors};

#[allow(clippy::module_name_repetitions)]
pub enum ApiError {
    JsonApi(jsonapi::Error),
    Sqlx(sqlx::Error),
}

impl From<jsonapi::Error> for ApiError {
    fn from(error: jsonapi::Error) -> Self {
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
                Json(Document::<()> {
                    data: None,
                    errors: Errors(vec![error]).into(),
                    links: None,
                }),
            )
                .into_response(),
            ApiError::Sqlx(error) => {
                tracing::error!(error =? error, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Document::<()> {
                        data: None,
                        errors: Errors(vec![jsonapi::Error {
                            status: 500,
                            source: None,
                            title: "Internal Server Error".to_string().into(),
                            detail: "An unexpected error has occurred".to_string().into(),
                        }])
                        .into(),
                        links: None,
                    }),
                )
                    .into_response()
            }
        }
    }
}
