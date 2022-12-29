use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LobbyAttributes {
    #[serde(rename = "name")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserAttributes {
    #[serde(rename = "name")]
    pub name: Option<String>,
}
