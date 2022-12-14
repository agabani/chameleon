use std::str::FromStr;

use uuid::Uuid;

use super::{LocalId, User, UserId};

pub struct Database;

impl Database {
    pub async fn find_or_create_user_id<C>(
        local_id: &LocalId,
        c: &mut C,
    ) -> Result<UserId, redis::RedisError>
    where
        C: redis::aio::ConnectionLike,
    {
        let key = user_id_by_local_id(local_id);

        let user_id: String = if let Some(user_id) = redis::Cmd::get(&key).query_async(c).await? {
            user_id
        } else {
            redis::Cmd::set(&key, Uuid::new_v4().to_string())
                .query_async(c)
                .await?;
            redis::Cmd::get(&key).query_async(c).await?
        };

        Ok(UserId::from_str(&user_id).expect("Failed to parse user id"))
    }

    pub async fn get_user<C>(user_id: UserId, c: &mut C) -> Result<Option<User>, redis::RedisError>
    where
        C: redis::aio::ConnectionLike,
    {
        let user: Option<String> = redis::Cmd::get(user_by_user_id(user_id))
            .query_async(c)
            .await?;

        let Some(user) = user else {
            return Ok(None)
        };

        let user = serde_json::from_str(&user).expect("Failed to deserialize user");
        Ok(Some(user))
    }

    pub async fn update_user<C>(user: &User, c: &mut C) -> Result<(), redis::RedisError>
    where
        C: redis::aio::ConnectionLike,
    {
        let json = serde_json::to_string(user).expect("Failed to serialize user");

        redis::Cmd::set(user_by_user_id(user.id()), json)
            .query_async(c)
            .await?;

        Ok(())
    }
}

fn user_id_by_local_id(local_id: &LocalId) -> String {
    format!("local_id:{}:user_id", local_id.value())
}

fn user_by_user_id(user_id: UserId) -> String {
    format!("user:user_id:{}", user_id.value())
}
