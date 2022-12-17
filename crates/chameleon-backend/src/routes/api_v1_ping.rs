use std::net::SocketAddr;

use crate::{domain::AuthenticationId, error::ApiError};

use axum::{
    extract::ConnectInfo,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[allow(clippy::unused_async)] // reason = "required by `axum::Router`"
#[tracing::instrument]
pub async fn get(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    authentication_id: AuthenticationId,
) -> Result<Response, ApiError> {
    Ok(Json(json!({})).into_response())
}
