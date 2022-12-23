use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    #[serde(rename = "sub")]
    pub sub: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Error {
    #[serde(rename = "error")]
    pub error: String,

    #[serde(rename = "error_description")]
    pub error_description: String,
}
