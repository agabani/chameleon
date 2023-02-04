use crate::{app::AppState, domain::local_id, error::ApiError};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;

pub const PATH: &str = "/api/v1/ping";

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handle))
}

#[allow(clippy::unused_async)] // reason = "required by `axum::Router`"
#[tracing::instrument]
async fn handle(local_id: local_id::LocalId) -> Result<Response, ApiError> {
    Ok((StatusCode::OK, Json(json!({}))).into_response())
}
