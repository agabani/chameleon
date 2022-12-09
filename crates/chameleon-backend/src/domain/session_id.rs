use uuid::Uuid;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SessionId(Uuid);

impl std::str::FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::from_str(s)?))
    }
}
