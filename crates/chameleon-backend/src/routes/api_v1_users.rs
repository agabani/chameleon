use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http;

use crate::{
    domain::{AuthenticationId, Database, UserId},
    error::ApiError,
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(mut state): State<AppState>,
    authentication_id: AuthenticationId,
    Path(user_id): Path<UserId>,
) -> Result<Response, ApiError> {
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
