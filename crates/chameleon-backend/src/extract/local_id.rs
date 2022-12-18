use core::{future::Future, marker::Send, pin::Pin};
use std::str::FromStr;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use crate::domain::LocalId;

impl<S> FromRequestParts<S> for LocalId {
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _: &'life1 S,
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

            LocalId::from_str(header).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Header of type `x-chameleon-local-id` was malformed",
                )
            })
        })
    }
}
