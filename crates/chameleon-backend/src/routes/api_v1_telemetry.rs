use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chameleon_protocol::http::TelemetryLevel;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{domain::LocalId, error::ApiError};

#[allow(clippy::unused_async)] // reason = "required by `axum::Router`"
#[instrument]
pub async fn post(
    local_id: LocalId,
    query: Query<PostQuery>,
    json: Json<Value>,
) -> Result<Response, ApiError> {
    match query.level {
        TelemetryLevel::Trace => trace!({json =? json}, "telemetry"),
        TelemetryLevel::Debug => debug!({json =? json}, "telemetry"),
        TelemetryLevel::Info => info!({json =? json}, "telemetry"),
        TelemetryLevel::Warn => warn!({json =? json}, "telemetry"),
        TelemetryLevel::Error => error!({ json =? json }, "telemetry"),
    };

    Ok(StatusCode::OK.into_response())
}

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    #[serde(rename = "level")]
    level: TelemetryLevel,
}
