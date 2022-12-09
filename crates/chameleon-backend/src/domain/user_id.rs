use uuid::Uuid;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }

    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::str::FromStr for UserId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::from_str(s)?))
    }
}
