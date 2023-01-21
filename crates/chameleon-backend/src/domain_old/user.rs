use crate::domain::UserId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
}
