use crate::domain::{LocalId, SessionId};

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[allow(clippy::unused_async)]
#[tracing::instrument]
pub async fn get(local_id: LocalId, session_id: SessionId) -> Response {
    tracing::info!("request");

    Json(json!({})).into_response()
}
