use core::{future::Future, marker::Send, pin::Pin};

use axum::{extract::FromRequestParts, http::request::Parts};
use chameleon_protocol::jsonapi::{self, Source};

use crate::{
    database::Database,
    domain::{LocalId, UserId},
    error::ApiError,
    AppState,
};

impl FromRequestParts<AppState> for UserId {
    type Rejection = ApiError;

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

            Database::select_user_id_by_local_id(&state.postgres_pool, local_id)
                .await?
                .ok_or_else(|| {
                    ApiError::JsonApi(jsonapi::Error {
                        status: 401,
                        source: Source {
                            header: "x-chameleon-local-id".to_string().into(),
                            parameter: None,
                            pointer: None,
                        }
                        .into(),
                        title: "Invalid Header".to_string().into(),
                        detail: "`x-chameleon-local-id` does not have a user"
                            .to_string()
                            .into(),
                    })
                })
        })
    }
}
