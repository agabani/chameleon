use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{domain::AuthenticationId, error::ApiError};

#[allow(clippy::unused_async)] // reason = "required by `axum::Router`"
#[instrument]
pub async fn post(
    authentication_id: AuthenticationId,
    query: Query<PostQuery>,
    json: Json<Value>,
) -> Result<Response, ApiError> {
    match query.level {
        Level::Trace => trace!({json =? json}, "telemetry"),
        Level::Debug => debug!({json =? json}, "telemetry"),
        Level::Info => info!({json =? json}, "telemetry"),
        Level::Warn => warn!({json =? json}, "telemetry"),
        Level::Error => error!({ json =? json }, "telemetry"),
    };

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    #[serde(rename = "level")]
    level: Level,
}

#[derive(Debug, Deserialize)]
pub enum Level {
    #[serde(rename = "trace")]
    Trace,

    #[serde(rename = "debug")]
    Debug,

    #[serde(rename = "info")]
    Info,

    #[serde(rename = "warn")]
    Warn,

    #[serde(rename = "error")]
    Error,
}
