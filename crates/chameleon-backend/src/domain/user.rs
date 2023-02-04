use super::{local_id::LocalId, user_id::UserId};

pub struct User {
    pub id: UserId,
    pub name: String,
}

impl User {
    pub fn signup(actor: LocalId, name: &str) -> Result<(User, Vec<Events>), CreateError> {
        let user = User {
            id: UserId::random(),
            name: name.to_string(),
        };

        let events = vec![
            Events::Created(CreatedEvent {
                name: user.name.clone(),
            }),
            Events::Linked(actor),
        ];

        Ok((user, events))
    }

    pub fn update(
        &mut self,
        actor: UserId,
        name: Option<&str>,
    ) -> Result<Vec<Events>, UpdateError> {
        if actor != self.id {
            return Err(UpdateError::NotOwner);
        }

        if let Some(name) = name {
            self.name = name.to_string();
        }

        Ok(vec![Events::Updated(UpdatedEvent {
            name: self.name.clone(),
        })])
    }
}

pub enum Events {
    Created(CreatedEvent),
    Linked(LocalId),
    Updated(UpdatedEvent),
}

pub struct CreatedEvent {
    pub name: String,
}

pub struct UpdatedEvent {
    pub name: String,
}

pub enum CreateError {}

pub enum UpdateError {
    NotOwner,
}
