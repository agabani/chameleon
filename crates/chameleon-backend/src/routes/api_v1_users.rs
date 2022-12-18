use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{database::Database, domain::UserId, error::ApiError, AppState};

#[tracing::instrument(skip(state))]
pub async fn get(
    State(state): State<AppState>,
    user_id: UserId,
    Path(id): Path<UserId>,
) -> Result<Response, ApiError> {
    if let Some(user) = Database::get_user_by_id(id, &state.postgres_pool).await? {
        let response = http::UserResponse {
            id: user.id().as_string(),
            name: user.name().to_string(),
        };
        Ok((StatusCode::OK, Json(response)).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}
