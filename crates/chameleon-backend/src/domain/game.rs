use super::UserId;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct GameId(pub uuid::Uuid);

pub struct Game {
    pub id: GameId,
    pub name: String,
    pub host: UserId,
}

impl GameId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
