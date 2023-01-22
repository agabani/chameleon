use super::{LobbyId, UserId};

pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub host: UserId,
    pub members: Vec<UserId>,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}

impl Lobby {
    /// Join a lobby.
    pub fn join(
        &mut self,
        user_id: UserId,
        passcode: &Option<String>,
    ) -> Result<Vec<Events>, JoinError> {
        let mut events = Vec::new();

        if self.require_passcode && &self.passcode != passcode {
            return Err(JoinError::IncorrectPasscode);
        }

        if self.members.contains(&user_id) {
            return Err(JoinError::AlreadyJoined);
        }

        self.members.push(user_id);
        events.push(Events::Joined(user_id));

        Ok(events)
    }

    /// Leave a lobby.
    pub fn leave(&mut self, user_id: UserId) -> Result<Vec<Events>, LeaveError> {
        let mut events = Vec::new();

        let Some((index, _)) = self
            .members
            .iter()
            .enumerate()
            .find(|member| *member.1 == user_id) else {
                return Err(LeaveError::NotMember)
            };

        self.members.remove(index);
        events.push(Events::Left(user_id));

        if self.members.is_empty() {
            events.push(Events::Empty);
            return Ok(events);
        }

        if self.host == user_id {
            self.host = *self.members.first().unwrap();
            events.push(Events::HostGranted(self.host));
        }

        Ok(events)
    }
}

pub enum Events {
    Empty,
    HostGranted(UserId),
    HostRevoked(UserId),
    Joined(UserId),
    Left(UserId),
}

pub enum JoinError {
    AlreadyJoined,
    IncorrectPasscode,
}

pub enum LeaveError {
    NotMember,
}
