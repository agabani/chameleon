use crate::domain::{LobbyId, UserId};

#[allow(clippy::module_name_repetitions)]
pub struct LobbyOld {
    pub id: LobbyId,
    pub name: String,
    pub host: UserId,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}
