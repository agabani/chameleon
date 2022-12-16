use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{
    domain::{AuthenticationId, Database, User},
    error::ApiError,
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(
    State(mut state): State<AppState>,
    authentication_id: AuthenticationId,
) -> Result<Response, ApiError> {
    if let Some(user) =
        Database::get_user(authentication_id.user_id(), &mut state.redis_connection).await?
    {
        let response = http::UserResponse {
            id: user.id().as_string(),
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
    authentication_id: AuthenticationId,
    Json(payload): Json<http::UserRequest>,
) -> Result<Response, ApiError> {
    Database::update_user(
        &User::new(authentication_id.user_id(), payload.name),
        &mut state.redis_connection,
    )
    .await?;

    Ok((StatusCode::NO_CONTENT).into_response())
}
