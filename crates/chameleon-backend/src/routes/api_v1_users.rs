use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{
    domain::{Database, LocalId, SessionId, UserId},
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(
    State(mut state): State<AppState>,
    local_id: LocalId,
    session_id: SessionId,
    Path(user_id): Path<String>,
) -> Response {
    tracing::info!("request");

    let user_id = match UserId::from_str(&user_id) {
        Ok(user_id) => user_id,
        Err(err) => {
            tracing::warn!(err =? err, "Invalid user id");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let user = match Database::get_user(user_id, &mut state.redis_connection).await {
        Ok(user) => user,
        Err(err) => {
            tracing::error!(err =? err, "Failed to get user");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match user {
        Some(user) => (
            StatusCode::OK,
            Json(http::UserResponse {
                id: user_id.as_string(),
                name: user.name().to_string(),
            }),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
