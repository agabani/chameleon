use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{
    domain::{Database, LocalId, SessionId, UserId},
    error::ApiError,
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(
    State(mut state): State<AppState>,
    local_id: LocalId,
    session_id: SessionId,
    Path(user_id): Path<UserId>,
) -> Result<Response, ApiError> {
    let user = Database::get_user(user_id, &mut state.redis_connection).await?;

    Ok(match user {
        Some(user) => (
            StatusCode::OK,
            Json(http::UserResponse {
                id: user_id.as_string(),
                name: user.name().to_string(),
            }),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    })
}
