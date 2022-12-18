use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http::UserRequest;

use crate::{
    database::Database,
    domain::{LocalId, User, UserId},
    error::ApiError,
    AppState,
};

#[tracing::instrument(skip(state))]
pub async fn post(
    State(state): State<AppState>,
    local_id: LocalId,
    Json(json): Json<UserRequest>,
) -> Result<Response, ApiError> {
    let conn = &state.postgres_pool;

    if Database::get_user_id_by_local_id(local_id, conn)
        .await?
        .is_some()
    {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Header of type `x-chameleon-local-id` was already associated with a user",
        )
            .into_response());
    }

    let user = User::new(UserId::random(), json.name);
    Database::save_user(&user, &state.postgres_pool).await?;
    Database::save_local_id(user.id(), local_id, &state.postgres_pool).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
