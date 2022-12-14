use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{
    domain::{Database, LocalId, SessionId, User},
    error::ApiError,
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(
    State(mut state): State<AppState>,
    local_id: LocalId,
    session_id: SessionId,
) -> Result<Response, ApiError> {
    let user_id = Database::find_or_create_user_id(&local_id, &mut state.redis_connection).await?;

    if let Some(user) = Database::get_user(user_id, &mut state.redis_connection).await? {
        let response = http::UserResponse {
            id: user_id.as_string(),
            name: user.name().to_string(),
        };
        Ok((StatusCode::OK, Json(response)).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

#[tracing::instrument(skip(state))]
pub async fn put(
    State(mut state): State<AppState>,
    local_id: LocalId,
    session_id: SessionId,
    Json(payload): Json<http::UserRequest>,
) -> Result<Response, ApiError> {
    let user_id = Database::find_or_create_user_id(&local_id, &mut state.redis_connection).await?;

    Database::update_user(
        &User::new(user_id, payload.name),
        &mut state.redis_connection,
    )
    .await?;

    Ok((StatusCode::NO_CONTENT).into_response())
}
