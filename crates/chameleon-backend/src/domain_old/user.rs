#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserId(pub uuid::Uuid);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

impl UserId {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
