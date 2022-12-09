#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Request {
    #[serde(rename = "authenticate")]
    Authenticate(AuthenticateRequest),
}

impl ToString for Request {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}

impl std::str::FromStr for Request {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AuthenticateRequest {
    #[serde(rename = "localId")]
    pub local_id: String,

    #[serde(rename = "sessionId")]
    pub session_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Response {
    #[serde(rename = "authenticated")]
    Authenticated,

    #[serde(rename = "message")]
    Message(MessageResponse),
}

impl ToString for Response {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}

impl std::str::FromStr for Response {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MessageResponse {
    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "userName")]
    pub user_name: String,

    #[serde(rename = "content")]
    pub content: String,
}
