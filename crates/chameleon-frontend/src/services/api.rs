use chameleon_protocol::http;
use gloo::net::{http::Request, Error};

use super::storage::{local_id, session_id};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ApiService {}

impl ApiService {
    pub async fn post_message(&self, request: &http::MessageRequest) -> Result<(), Error> {
        let response = Request::post("/api/v1/message")
            .authentication_headers()
            .json(request)?
            .send()
            .await?;

        match response.status() {
            200 => Ok(()),
            status => todo!("Unexpected status code: {status}"),
        }
    }

    #[allow(dead_code)]
    pub async fn ping(&self) -> Result<(), Error> {
        let response = Request::get("/api/v1/ping")
            .authentication_headers()
            .send()
            .await?;

        match response.status() {
            200 => Ok(()),
            status => todo!("Unexpected status code: {status}"),
        }
    }

    pub async fn get_user(&self) -> Result<Option<http::UserResponse>, Error> {
        let response = Request::get("/api/v1/user")
            .authentication_headers()
            .send()
            .await?;

        match response.status() {
            200 => {
                let user = response.json().await?;
                Ok(Some(user))
            }
            404 => Ok(None),
            status => todo!("Unexpected status code: {status}"),
        }
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<http::UserResponse>, Error> {
        let response = Request::get(&format!("/api/v1/users/{}", user_id))
            .authentication_headers()
            .send()
            .await?;

        match response.status() {
            200 => {
                let user = response.json().await?;
                Ok(Some(user))
            }
            404 => Ok(None),
            status => todo!("Unexpected status code: {status}"),
        }
    }

    pub async fn put_user(&self, user: &http::UserRequest) -> Result<(), Error> {
        let response = Request::put("/api/v1/user")
            .authentication_headers()
            .json(user)?
            .send()
            .await?;

        match response.status() {
            204 => Ok(()),
            status => todo!("Unexpected status code: {status}"),
        }
    }
}

trait RequestExt {
    fn authentication_headers(self) -> Self;
}

impl RequestExt for Request {
    fn authentication_headers(self) -> Self {
        self.header(
            "x-chameleon-local-id",
            &local_id().expect("Failed to get local id"),
        )
        .header(
            "x-chameleon-session-id",
            &session_id().expect("Failed to get session id"),
        )
    }
}
