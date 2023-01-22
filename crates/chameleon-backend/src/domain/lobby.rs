use super::{LobbyId, UserId};

pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub members: Vec<Member>,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}

pub struct Member {
    pub host: bool,
    pub user_id: UserId,
}

impl Lobby {
    pub fn create(
        name: String,
        actor: UserId,
        passcode: Option<String>,
        require_passcode: bool,
    ) -> Result<(Self, Vec<Events>), CreateError> {
        if require_passcode && passcode.is_none() {
            return Err(CreateError::MissingPasscode);
        }

        let this = Self {
            id: LobbyId::random(),
            name: name.clone(),
            members: vec![Member {
                host: true,
                user_id: actor,
            }],
            passcode: passcode.clone(),
            require_passcode,
        };

        Ok((
            this,
            vec![
                Events::Created(CreatedEvent {
                    name,
                    passcode,
                    require_passcode,
                }),
                Events::Joined(actor),
                Events::HostGranted(actor),
            ],
        ))
    }

    /// Join.
    pub fn join(
        &mut self,
        actor: UserId,
        passcode: &Option<String>,
    ) -> Result<Vec<Events>, JoinError> {
        if self.require_passcode && &self.passcode != passcode {
            return Err(JoinError::IncorrectPasscode);
        }

        if self.members.iter().any(|member| member.user_id == actor) {
            return Err(JoinError::AlreadyJoined);
        }

        self.members.push(Member {
            user_id: actor,
            host: false,
        });

        Ok(vec![Events::Joined(actor)])
    }

    /// Leave.
    pub fn leave(&mut self, actor: UserId) -> Result<Vec<Events>, LeaveError> {
        let Some((index, _)) = self
            .members
            .iter()
            .enumerate()
            .find(|member| member.1.user_id == actor) else {
                return Err(LeaveError::NotMember)
            };

        let mut events = Vec::new();

        let member = self.members.remove(index);
        events.push(Events::Left(actor));

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
        actor: UserId,
        message: String,
    ) -> Result<Vec<Events>, SendChatMessageError> {
        if !self.members.iter().any(|member| member.user_id == actor) {
            return Err(SendChatMessageError::NotMember);
        }

        Ok(vec![Events::ChatMessage(ChatMessageEvent {
            user_id: actor,
            message,
        })])
    }

    /// Update.
    pub fn update(
        &mut self,
        actor: UserId,
        name: &Option<String>,
        passcode: &Option<String>,
        require_passcode: Option<bool>,
    ) -> Result<Vec<Events>, UpdateError> {
        if !self
            .members
            .iter()
            .any(|member| member.host && member.user_id == actor)
        {
            return Err(UpdateError::NotHost);
        }

        {
            let require_password = match require_passcode {
                Some(require_passcode) => require_passcode,
                None => self.require_passcode,
            };
            let passcode = self.passcode.is_some() || passcode.is_some();
            if require_password && !passcode {
                return Err(UpdateError::MissingPasscode);
            }
        }

        if let Some(name) = &name {
            self.name = name.clone();
        }

        if let Some(passcode) = &passcode {
            self.passcode = Some(passcode.clone());
        }

        if let Some(require_passcode) = require_passcode {
            self.require_passcode = require_passcode;
        }

        Ok(vec![Events::Updated(UpdatedEvent {
            name: self.name.clone(),
            passcode: self.passcode.clone(),
            require_passcode: self.require_passcode,
        })])
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
    Updated(UpdatedEvent),
}

pub struct ChatMessageEvent {
    pub user_id: UserId,
    pub message: String,
}

pub struct CreatedEvent {
    pub name: String,
    pub passcode: Option<String>,
    pub require_passcode: bool,
}

pub struct UpdatedEvent {
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

pub enum UpdateError {
    MissingPasscode,
    NotHost,
}
