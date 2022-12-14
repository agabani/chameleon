use crate::{
    domain::{Database, LocalId, SessionId},
    error::ApiError,
    AppState,
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::{http, ws};

#[tracing::instrument(skip(state))]
pub async fn post(
    State(mut state): State<AppState>,
    local_id: LocalId,
    session_id: SessionId,
    Json(body): Json<http::MessageRequest>,
) -> Result<Response, ApiError> {
    let user_id = Database::find_or_create_user_id(&local_id, &mut state.redis_connection).await?;

    let user = Database::get_user(user_id, &mut state.redis_connection)
        .await?
        .expect("Failed to get user by id");

    let message = ws::Response::Message(ws::MessageResponse {
        user_id: user_id.as_string(),
        user_name: user.name().to_string(),
        content: body.content,
    });

    redis::Cmd::publish("testing", serde_json::to_string(&message).unwrap())
        .query_async(&mut state.redis_connection)
        .await?;

    Ok(StatusCode::OK.into_response())
}
