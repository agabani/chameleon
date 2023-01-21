#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct LocalId(pub uuid::Uuid);

impl std::str::FromStr for LocalId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::from_str(s)?))
    }
}
