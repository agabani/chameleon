use uuid::Uuid;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl std::str::FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Uuid::from_str(s)?))
    }
}
