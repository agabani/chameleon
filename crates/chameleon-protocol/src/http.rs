#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MessageRequest {
    #[serde(rename = "content")]
    pub content: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserResponse {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserRequest {
    #[serde(rename = "name")]
    pub name: String,
}
