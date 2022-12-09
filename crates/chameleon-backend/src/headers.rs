use std::str::FromStr;

use axum::{
    headers::{Header, HeaderName},
    http::HeaderValue,
};

use crate::domain::LocalId;
use uuid::Uuid;

static X_CHAMELEON_LOCAL_ID: HeaderName = HeaderName::from_static("x-chameleon-local-id");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XChameleonLocalId(pub HeaderValue);

impl Header for XChameleonLocalId {
    fn name() -> &'static HeaderName {
        &X_CHAMELEON_LOCAL_ID
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .cloned()
            .ok_or_else(axum::headers::Error::invalid)
            .map(Self)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(::std::iter::once((&self.0).into()));
    }
}

impl TryFrom<XChameleonLocalId> for LocalId {
    type Error = uuid::Error;

    fn try_from(value: XChameleonLocalId) -> Result<Self, Self::Error> {
        let s = value.0.to_str().expect("Failed to read header value");
        Ok(LocalId::new(Uuid::from_str(s)?))
    }
}

static X_CHAMELEON_SESSION_ID: HeaderName = HeaderName::from_static("x-chameleon-session-id");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XChameleonSessionId(pub HeaderValue);

impl Header for XChameleonSessionId {
    fn name() -> &'static HeaderName {
        &X_CHAMELEON_SESSION_ID
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .cloned()
            .ok_or_else(axum::headers::Error::invalid)
            .map(Self)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(::std::iter::once((&self.0).into()));
    }
}
