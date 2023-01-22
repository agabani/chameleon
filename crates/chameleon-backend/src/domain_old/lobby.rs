use crate::domain::{LobbyId, UserId};

pub struct LobbyOld {
    pub id: LobbyId,
    pub name: String,
    pub host: UserId,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}
