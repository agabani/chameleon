use core::{future::Future, marker::Send, pin::Pin};
use std::str::FromStr;

use axum::{extract::FromRequestParts, http::request::Parts};
use chameleon_protocol::jsonapi::{self, Source};

use crate::{domain::LocalId, error::ApiError};

impl<S> FromRequestParts<S> for LocalId {
    type Rejection = ApiError;

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
                .ok_or_else(|| {
                    ApiError::JsonApi(
                        jsonapi::Error {
                            status: 400,
                            source: Source {
                                header: "x-chameleon-local-id".to_string().into(),
                                parameter: None,
                                pointer: None,
                            }
                            .into(),
                            title: "Invalid Header".to_string().into(),
                            detail: "`x-chameleon-local-id` must be present".to_string().into(),
                        }
                        .into(),
                    )
                })?;

            LocalId::from_str(header).map_err(|error| {
                ApiError::JsonApi(
                    jsonapi::Error {
                        status: 400,
                        source: Source {
                            header: "x-chameleon-local-id".to_string().into(),
                            parameter: None,
                            pointer: None,
                        }
                        .into(),
                        title: "Invalid Header".to_string().into(),
                        detail: error.to_string().into(),
                    }
                    .into(),
                )
            })
        })
    }
}
