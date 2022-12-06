use gloo::net::{
    http::{Request, Response},
    Error,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ApiService {}

impl ApiService {
    #[allow(dead_code)]
    pub async fn ping(&self) -> Result<Response, Error> {
        Request::get("/api/v1/ping").send().await
    }

    pub async fn message(&self, body: &str) -> Result<Response, Error> {
        Request::post("/api/v1/message").body(body).send().await
    }
}
