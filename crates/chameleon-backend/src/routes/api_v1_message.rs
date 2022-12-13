use crate::{
    domain::{Database, LocalId, SessionId},
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
) -> Response {
    tracing::info!("request");

    let user_id =
        match Database::find_or_create_user_id(&local_id, &mut state.redis_connection).await {
            Ok(user_id) => user_id,
            Err(err) => {
                tracing::error!(err =? err, "Failed to find or create user id");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    let user = match Database::get_user(user_id, &mut state.redis_connection).await {
        Ok(user_id) => user_id.expect("Failed to get user by id"),
        Err(err) => {
            tracing::error!(err =? err, "Failed to get user by id");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let message = ws::Response::Message(ws::MessageResponse {
        user_id: user_id.as_string(),
        user_name: user.name().to_string(),
        content: body.content,
    });

    let message = serde_json::to_string(&message).unwrap();

    redis::Cmd::publish("testing", message)
        .query_async(&mut state.redis_connection)
        .await
        .map(|_: ()| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        .into_response()
}
