use super::{LobbyId, UserId};

pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub members: Vec<LobbyMember>,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}

pub struct LobbyMember {
    pub host: bool,
    pub user_id: UserId,
}

impl Lobby {
    pub fn create(
        name: String,
        host: UserId,
        passcode: Option<String>,
        require_passcode: bool,
    ) -> Result<(Self, Vec<Events>), CreateError> {
        let mut events = Vec::new();

        if require_passcode && passcode.is_none() {
            return Err(CreateError::MissingPasscode);
        }

        let this = Self {
            id: LobbyId::random(),
            name: name.clone(),
            members: vec![LobbyMember {
                host: true,
                user_id: host,
            }],
            passcode: passcode.clone(),
            require_passcode,
        };

        events.push(Events::Created(CreatedEvent {
            id: this.id,
            name,
            passcode,
            require_passcode,
        }));
        events.push(Events::Joined(host));
        events.push(Events::HostGranted(host));

        Ok((this, events))
    }

    /// Join.
    pub fn join(
        &mut self,
        user_id: UserId,
        passcode: &Option<String>,
    ) -> Result<Vec<Events>, JoinError> {
        let mut events = Vec::new();

        if self.require_passcode && &self.passcode != passcode {
            return Err(JoinError::IncorrectPasscode);
        }

        if self
            .members
            .iter()
            .find(|member| member.user_id == user_id)
            .is_some()
        {
            return Err(JoinError::AlreadyJoined);
        }

        self.members.push(LobbyMember {
            user_id,
            host: false,
        });
        events.push(Events::Joined(user_id));

        Ok(events)
    }

    /// Leave.
    pub fn leave(&mut self, user_id: UserId) -> Result<Vec<Events>, LeaveError> {
        let mut events = Vec::new();

        let Some((index, _)) = self
            .members
            .iter()
            .enumerate()
            .find(|member| member.1.user_id == user_id) else {
                return Err(LeaveError::NotMember)
            };

        let member = self.members.remove(index);
        events.push(Events::Left(user_id));

        if self.members.is_empty() {
            events.push(Events::Empty);
            return Ok(events);
        }

        if member.host {
            let member = self.members.first_mut().unwrap();
            member.host = true;
            events.push(Events::HostGranted(member.user_id));
        }

        Ok(events)
    }

    /// Send chat message.
    pub fn send_chat_message(
        &mut self,
        user_id: UserId,
        message: String,
    ) -> Result<Vec<Events>, SendChatMessageError> {
        let mut events = Vec::new();

        if !self
            .members
            .iter()
            .find(|member| member.user_id == user_id)
            .is_none()
        {
            return Err(SendChatMessageError::NotMember);
        }

        events.push(Events::ChatMessage(ChatMessageEvent { user_id, message }));

        Ok(events)
    }
}

pub enum Events {
    ChatMessage(ChatMessageEvent),
    Created(CreatedEvent),
    Empty,
    HostGranted(UserId),
    HostRevoked(UserId),
    Joined(UserId),
    Left(UserId),
}

pub struct ChatMessageEvent {
    pub user_id: UserId,
    pub message: String,
}

pub struct CreatedEvent {
    pub id: LobbyId,
    pub name: String,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}

pub enum CreateError {
    MissingPasscode,
}

pub enum JoinError {
    AlreadyJoined,
    IncorrectPasscode,
}

pub enum LeaveError {
    NotMember,
}

pub enum SendChatMessageError {
    NotMember,
}
