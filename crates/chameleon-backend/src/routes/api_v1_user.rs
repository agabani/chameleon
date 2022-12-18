use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{
    database::Database,
    domain::{User, UserId},
    error::ApiError,
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(State(state): State<AppState>, user_id: UserId) -> Result<Response, ApiError> {
    if let Some(user) = Database::get_user_by_id(user_id, &state.postgres_pool).await? {
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
    State(state): State<AppState>,
    user_id: UserId,
    Json(payload): Json<http::UserRequest>,
) -> Result<Response, ApiError> {
    Database::save_user(&User::new(user_id, payload.name), &state.postgres_pool).await?;
    Ok((StatusCode::NO_CONTENT).into_response())
}
