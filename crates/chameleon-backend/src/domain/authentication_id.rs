use super::{LocalId, SessionId, UserId};

#[derive(Debug, Clone, Copy)]
pub struct AuthenticationId {
    #[allow(dead_code)] // reason = "used in extractor `AuthenticationId` extractor"
    local_id: LocalId,

    #[allow(dead_code)] // reason = "used in extractor `AuthenticationId` extractor"
    session_id: SessionId,

    user_id: UserId,
}

impl AuthenticationId {
    pub fn new(local_id: LocalId, session_id: SessionId, user_id: UserId) -> Self {
        Self {
            local_id,
            session_id,
            user_id,
        }
    }

    pub fn local_id(&self) -> LocalId {
        self.local_id
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}
