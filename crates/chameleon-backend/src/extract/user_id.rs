use core::{future::Future, marker::Send, pin::Pin};

use axum::{extract::FromRequestParts, http::request::Parts};
use chameleon_protocol::jsonapi::{self, Source};

use crate::{
    app::AppState,
    database::Database,
    domain::{local_id, user_id},
    error::ApiError,
};

impl FromRequestParts<AppState> for user_id::UserId {
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
            let local_id = local_id::LocalId::from_request_parts(parts, state).await?;

            Database::select_user_id_by_local_id(&state.pool, local_id)
                .await?
                .ok_or_else(|| {
                    ApiError::JsonApi(Box::new(jsonapi::Error {
                        status: 401,
                        source: Some(Source {
                            header: Some("x-chameleon-local-id".to_string()),
                            parameter: None,
                            pointer: None,
                        }),
                        title: Some("Invalid Header".to_string()),
                        detail: Some("`x-chameleon-local-id` does not have a user".to_string()),
                    }))
                })
        })
    }
}
