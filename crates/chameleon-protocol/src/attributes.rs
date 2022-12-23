use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameAttributes {
    #[serde(rename = "name")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserAttributes {
    #[serde(rename = "name")]
    pub name: Option<String>,
}
