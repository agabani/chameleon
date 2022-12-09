use crate::headers::{XChameleonLocalId, XChameleonSessionId};

use axum::{
    response::{IntoResponse, Response},
    Json, TypedHeader,
};
use serde_json::json;

#[allow(clippy::unused_async)]
#[tracing::instrument]
pub async fn get(
    TypedHeader(local_id): TypedHeader<XChameleonLocalId>,
    TypedHeader(session_id): TypedHeader<XChameleonSessionId>,
) -> Response {
    tracing::info!("request");

    Json(json!({})).into_response()
}
