use super::UserId;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct LobbyId(pub uuid::Uuid);

pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub host: UserId,
}

impl LobbyId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
