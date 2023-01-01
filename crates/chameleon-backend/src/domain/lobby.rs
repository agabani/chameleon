use super::UserId;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct LobbyId(pub uuid::Uuid);

pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub host: UserId,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}

impl LobbyId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
