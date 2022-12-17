use std::net::SocketAddr;

use crate::{
    domain::{AuthenticationId, Database},
    error::ApiError,
    AppState,
};

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::{http, ws};

#[tracing::instrument(skip(state))]
pub async fn post(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(mut state): State<AppState>,
    authentication_id: AuthenticationId,
    Json(body): Json<http::MessageRequest>,
) -> Result<Response, ApiError> {
    let user = Database::get_user(authentication_id.user_id(), &mut state.redis_connection)
        .await?
        .expect("Failed to get user by id");

    let message = ws::Response::Message(ws::MessageResponse {
        user_id: user.id().as_string(),
        user_name: user.name().to_string(),
        content: body.content,
    });

    Database::publish_message(message, &mut state.redis_connection).await?;

    Ok(StatusCode::OK.into_response())
}
