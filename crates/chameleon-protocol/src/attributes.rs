use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ChatMessageAttributes {
    #[serde(rename = "message")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LobbyAttributes {
    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "passcode")]
    pub passcode: Option<String>,

    #[serde(rename = "require_passcode")]
    pub require_passcode: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserAttributes {
    #[serde(rename = "name")]
    pub name: Option<String>,
}
