use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json, TypedHeader,
};
use chameleon_protocol::http;

use crate::{
    domain::{Database, User},
    headers::{XChameleonLocalId, XChameleonSessionId},
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn get(
    State(mut state): State<AppState>,
    TypedHeader(local_id): TypedHeader<XChameleonLocalId>,
    TypedHeader(session_id): TypedHeader<XChameleonSessionId>,
) -> Response {
    let local_id = match local_id.try_into() {
        Ok(local_id) => local_id,
        Err(err) => {
            tracing::warn!(err =? err, "Invalid local id");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let user_id =
        match Database::find_or_create_user_id(&local_id, &mut state.redis_connection).await {
            Ok(user_id) => user_id,
            Err(err) => {
                tracing::error!(err =? err, "Failed to find or create user id");
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

#[tracing::instrument(skip(state))]
pub async fn put(
    State(mut state): State<AppState>,
    TypedHeader(local_id): TypedHeader<XChameleonLocalId>,
    TypedHeader(session_id): TypedHeader<XChameleonSessionId>,
    Json(payload): Json<http::UserRequest>,
) -> Response {
    tracing::info!("Request");

    let local_id = match local_id.try_into() {
        Ok(local_id) => local_id,
        Err(err) => {
            tracing::warn!(err =? err, "Invalid local id");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let user_id =
        match Database::find_or_create_user_id(&local_id, &mut state.redis_connection).await {
            Ok(user_id) => user_id,
            Err(err) => {
                tracing::error!(err =? err, "Failed to find or create user id");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    if let Err(err) = Database::update_user(
        &User::new(user_id, payload.name),
        &mut state.redis_connection,
    )
    .await
    {
        tracing::error!(user_id =? user_id, err =? err, "Failed to user id");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    (StatusCode::NO_CONTENT).into_response()
}
