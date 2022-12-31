use serde::{Deserialize, Serialize};

use crate::jsonrpc::Frame;

pub type LobbyFrame = Frame<LobbyRequest, LobbyResponse>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "method", content = "params")]
pub enum LobbyRequest {
    #[serde(rename = "authenticate")]
    Authenticate(LobbyAuthenticate),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LobbyAuthenticate {
    #[serde(rename = "localId", skip_serializing_if = "Option::is_none")]
    pub local_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "method", content = "value")]
pub enum LobbyResponse {
    #[serde(rename = "authenticate")]
    Authenticate(bool),
}
