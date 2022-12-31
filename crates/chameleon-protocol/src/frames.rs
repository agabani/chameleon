use serde::{Deserialize, Serialize};

use crate::jsonrpc::Frame;

pub type LobbyFrame = Frame<LobbyRequest, LobbyResponse>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "method", content = "params")]
pub enum LobbyRequest {
    #[serde(rename = "authenticate")]
    Authenticate(LobbyAuthenticate),

    #[serde(rename = "chat_message")]
    ChatMessage(LobbyChatMessage),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LobbyAuthenticate {
    #[serde(rename = "local_id", skip_serializing_if = "Option::is_none")]
    pub local_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LobbyChatMessage {
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "method", content = "value")]
pub enum LobbyResponse {
    #[serde(rename = "authenticate")]
    Authenticate(bool),
}
