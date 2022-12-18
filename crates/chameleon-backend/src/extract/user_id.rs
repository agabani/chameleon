use core::{future::Future, marker::Send, pin::Pin};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::{
    database::Database,
    domain::{LocalId, UserId},
    AppState,
};

impl FromRequestParts<AppState> for UserId {
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
            let local_id = LocalId::from_request_parts(parts, state).await?;

            Database::get_user_id_by_local_id(local_id, &state.postgres_pool)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error has occurred",
                    )
                })?
                .ok_or((
                    StatusCode::UNAUTHORIZED,
                    "Header of type `x-chameleon-local-id` does not have a user",
                ))
        })
    }
}
