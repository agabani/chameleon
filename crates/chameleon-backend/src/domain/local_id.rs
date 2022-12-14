use uuid::Uuid;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct LocalId(Uuid);

impl LocalId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl std::str::FromStr for LocalId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Uuid::from_str(s)?))
    }
}
