use chameleon_protocol::{
    attributes::{GameAttributes, UserAttributes},
    http,
    jsonapi::Document,
    openid_connect,
};
use gloo::{
    console,
    net::{http::Request, Error},
};

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

    pub async fn query_games(&self, url: Option<&str>) -> Result<Document<GameAttributes>, Error> {
        Request::get(url.unwrap_or("/api/v1/games"))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn create_user(
        &self,
        document: &Document<UserAttributes>,
    ) -> Result<Document<UserAttributes>, Error> {
        Request::post("/api/v1/users")
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_user(&self, id: &str) -> Result<Document<UserAttributes>, Error> {
        Request::get(&format!("/api/v1/users/{id}"))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn update_user(
        &self,
        id: &str,
        document: &Document<UserAttributes>,
    ) -> Result<Document<UserAttributes>, Error> {
        Request::patch(&format!("/api/v1/users/{id}"))
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_userinfo(&self) -> Result<Option<openid_connect::UserInfo>, Error> {
        let response = Request::get("/api/v1/userinfo")
            .authentication_headers()
            .send()
            .await?;

        match response.status() {
            200 => Ok(Some(response.json().await?)),
            401 => Ok(None),
            status => todo!("Unexpected status code: {status}"),
        }
    }

    pub async fn post_telemetry<T>(&self, value: &T, level: http::TelemetryLevel)
    where
        T: serde::Serialize,
    {
        let request = match Request::post("/api/v1/telemetry")
            .authentication_headers()
            .query([("level", level)])
            .json(value)
        {
            Ok(request) => request,
            Err(err) => {
                return console::error!(format!("{err:?}"), "Failed to build request");
            }
        };

        let response = match request.send().await {
            Ok(response) => response,
            Err(err) => {
                return console::error!(format!("{err:?}"), "Failed to send request");
            }
        };

        if response.status() != 200 {
            return console::error!(response.status(), "Unexpected status code");
        }

        match response.status() {
            200 => {}
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
