use crate::{database::Database, domain::UserId, error::ApiError, AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::{http, ws};

#[tracing::instrument(skip(state))]
pub async fn post(
    State(state): State<AppState>,
    user_id: UserId,
    Json(body): Json<http::MessageRequest>,
) -> Result<Response, ApiError> {
    let user = Database::get_user_by_id(user_id, &state.postgres_pool)
        .await?
        .expect("Failed to get user by id");

    let message = ws::Response::Message(ws::MessageResponse {
        user_id: user.id().as_string(),
        user_name: user.name().to_string(),
        content: body.content,
    });

    Database::notify(&state.postgres_pool, "testing", &message).await?;

    Ok(StatusCode::OK.into_response())
}
