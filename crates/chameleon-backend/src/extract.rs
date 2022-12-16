use core::{future::Future, marker::Send, pin::Pin};
use std::str::FromStr;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::{
    domain::{AuthenticationId, Database, LocalId, SessionId},
    AppState,
};

impl FromRequestParts<AppState> for AuthenticationId {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let header = parts
                .headers
                .get("x-chameleon-local-id")
                .and_then(|header| header.to_str().ok())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    "Header of type `x-chameleon-local-id` was missing",
                ))?;

            let local_id = LocalId::from_str(header).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Header of type `x-chameleon-local-id` was malformed",
                )
            })?;

            let header = parts
                .headers
                .get("x-chameleon-session-id")
                .and_then(|header| header.to_str().ok())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    "Header of type `x-chameleon-session-id` was missing",
                ))?;

            let session_id = SessionId::from_str(header).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Header of type `x-chameleon-session-id` was malformed",
                )
            })?;

            let mut redis_connection = state.redis_connection.clone();

            let user_id = Database::find_or_create_user_id(&local_id, &mut redis_connection)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error has occured",
                    )
                })?;

            Ok(AuthenticationId::new(local_id, session_id, user_id))
        })
    }
}
