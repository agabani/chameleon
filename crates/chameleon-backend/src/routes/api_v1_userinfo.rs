use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chameleon_protocol::openid_connect::{self, UserInfo};

use crate::{app::AppState, domain::user_id, error::ApiError};

pub const PATH: &str = "/api/v1/userinfo";

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handle))
}

#[allow(clippy::unused_async)] // reason = "required by `axum::Router`"
async fn handle(user_id: Option<user_id::UserId>) -> Result<Response, ApiError> {
    if let Some(user_id) = user_id {
        Ok((
            StatusCode::OK,
            Json(UserInfo {
                sub: user_id.0.to_string(),
            }),
        )
            .into_response())
    } else {
        Ok((
            StatusCode::UNAUTHORIZED,
            Json(openid_connect::Error {
                error: "invalid_request".to_string(),
                error_description: "Invalid Credentials".to_string(),
            }),
        )
            .into_response())
    }
}
