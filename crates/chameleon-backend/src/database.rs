use chameleon_protocol::{
    frames::{self, LobbyFrame, LobbyRequest},
    jsonapi::{self, Source},
};
use sqlx::{postgres::PgListener, Executor, Pool, Postgres};

use crate::domain::{lobby, lobby_id, local_id, user, user_id};

pub struct Database {}

impl Database {
    pub async fn listener(conn: &Pool<Postgres>) -> Result<PgListener, sqlx::Error> {
        PgListener::connect_with(conn).await
    }

    pub async fn query_lobby<'c, E>(
        conn: E,
        keyset_pagination: KeysetPagination,
    ) -> Result<(Vec<lobby::Query>, i64), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let records = sqlx::query!(
            r#"SELECT l.id, l.public_id, l.name, l.require_passcode
            FROM lobby l
            WHERE l.id > $1
            ORDER BY l.id
            LIMIT $2;"#,
            keyset_pagination.id,
            keyset_pagination.limit,
        )
        .fetch_all(conn)
        .await?;

        let last_record_id = records
            .last()
            .map_or(keyset_pagination.id, |record| record.id);

        let lobbies = records
            .into_iter()
            .map(|record| lobby::Query {
                id: lobby_id::LobbyId(record.public_id),
                name: record.name,
                require_passcode: record.require_passcode,
            })
            .collect();

        Ok((lobbies, last_record_id))
    }

    pub async fn query_lobby_member<'c, E>(
        conn: E,
        lobby_id: lobby_id::LobbyId,
        keyset_pagination: KeysetPagination,
    ) -> Result<(Vec<user::User>, i64), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let records = sqlx::query!(
            r#"SELECT lm.id, u.public_id, u.name
            FROM lobby l
                     JOIN lobby_member lm on l.id = lm.lobby_id
                     JOIN "user" u on u.id = lm.user_id
            WHERE l.public_id = $3
              AND lm.id > $1
            ORDER BY lm.id
            LIMIT $2;"#,
            keyset_pagination.id,
            keyset_pagination.limit,
            lobby_id.0
        )
        .fetch_all(conn)
        .await?;

        let last_record_id = records
            .last()
            .map_or(keyset_pagination.id, |record| record.id);

        let users = records
            .into_iter()
            .map(|record| user::User {
                id: user_id::UserId(record.public_id),
                name: record.name,
            })
            .collect();

        Ok((users, last_record_id))
    }

    pub async fn select_user_id_by_local_id<'c, E>(
        conn: E,
        local_id: local_id::LocalId,
    ) -> Result<Option<user_id::UserId>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT u.public_id
            FROM "user" u
                     JOIN local l ON u.id = l.user_id
            WHERE l.public_id = $1;"#,
            local_id.0,
        )
        .map(|record| user_id::UserId(record.public_id))
        .fetch_optional(conn)
        .await
    }

    pub async fn load_lobby<'c, E>(
        conn: E,
        lobby_id: lobby_id::LobbyId,
    ) -> Result<Option<lobby::Lobby>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres> + Copy,
    {
        let Some(lobby) = sqlx::query!(
            r#"SELECT l.public_id, l.name, l.passcode, l.require_passcode, u.public_id host_public_id
            FROM lobby l
                     JOIN lobby_member lm ON l.id = lm.lobby_id
                     JOIN "user" u ON u.id = lm.user_id
            WHERE l.public_id = $1
              AND lm.host IS TRUE;"#,
            lobby_id.0
        )
        .fetch_optional(conn)
        .await? else {
            return Ok(None);
        };

        let members = sqlx::query!(
            r#"SELECT u.public_id, lm.host
            FROM lobby_member lm
                     JOIN "user" u on lm.user_id = u.id
                     JOIN lobby l on lm.lobby_id = l.id
            WHERE l.public_id = $1;
            "#,
            lobby_id.0
        )
        .fetch_all(conn)
        .await?;

        Ok(Some(lobby::Lobby {
            id: lobby_id::LobbyId(lobby.public_id),
            name: lobby.name,
            members: members
                .into_iter()
                .map(|member| lobby::Member {
                    host: member.host,
                    user_id: user_id::UserId(member.public_id),
                })
                .collect(),
            passcode: lobby.passcode,
            require_passcode: lobby.require_passcode,
        }))
    }

    pub async fn load_user<'c, E>(
        conn: E,
        user_id: user_id::UserId,
    ) -> Result<Option<user::User>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT u.public_id, u.name
            FROM "user" u
            WHERE u.public_id = $1;"#,
            user_id.0
        )
        .fetch_optional(conn)
        .await
        .map(|record| {
            record.map(|record| user::User {
                id: user_id::UserId(record.public_id),
                name: record.name,
            })
        })
    }

    pub async fn save_lobby(
        pool: &Pool<Postgres>,
        lobby_id: lobby_id::LobbyId,
        events: &[lobby::Events],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = pool.begin().await?;

        for event in events {
            match event {
                lobby::Events::ChatMessage(chat_message) => {
                    Self::notify_lobby(
                        &mut transaction,
                        lobby_id,
                        frames::LobbyRequest::ChatMessage(frames::LobbyChatMessage {
                            user_id: Some(chat_message.user_id.0.to_string()),
                            message: Some(chat_message.message.clone()),
                        }),
                    )
                    .await?;
                }
                lobby::Events::Created(event) => {
                    Self::insert_lobby(
                        &mut transaction,
                        lobby_id,
                        &event.name,
                        &event.passcode,
                        event.require_passcode,
                    )
                    .await?;
                }
                lobby::Events::Empty => {
                    Self::delete_lobby(&mut transaction, lobby_id).await?;
                }
                lobby::Events::HostGranted(user_id) => {
                    Self::update_lobby_member_host(&mut transaction, lobby_id, *user_id, true)
                        .await?;
                }
                lobby::Events::HostRevoked(user_id) => {
                    Self::update_lobby_member_host(&mut transaction, lobby_id, *user_id, false)
                        .await?;
                }
                lobby::Events::Joined(user_id) => {
                    Self::insert_lobby_member(&mut transaction, lobby_id, *user_id).await?;
                    Self::notify_lobby(
                        &mut transaction,
                        lobby_id,
                        frames::LobbyRequest::UserJoined(frames::LobbyUserJoined {
                            user_id: Some(user_id.0.to_string()),
                        }),
                    )
                    .await?;
                }
                lobby::Events::Left(user_id) => {
                    Self::delete_lobby_member(&mut transaction, lobby_id, *user_id).await?;
                    Self::notify_lobby(
                        &mut transaction,
                        lobby_id,
                        frames::LobbyRequest::UserLeft(frames::LobbyUserLeft {
                            user_id: Some(user_id.0.to_string()),
                        }),
                    )
                    .await?;
                }
                lobby::Events::Updated(event) => {
                    Self::update_lobby(
                        &mut transaction,
                        lobby_id,
                        &event.name,
                        &event.passcode,
                        event.require_passcode,
                    )
                    .await?;
                }
            }
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_user(
        pool: &Pool<Postgres>,
        user_id: user_id::UserId,
        events: &[user::Events],
    ) -> Result<(), sqlx::Error> {
        let mut transaction = pool.begin().await?;

        for event in events {
            match event {
                user::Events::Created(event) => {
                    Self::insert_user(&mut transaction, user_id, &event.name).await?;
                }
                user::Events::Linked(local_id) => {
                    Self::insert_local(&mut transaction, *local_id, user_id).await?;
                }
                user::Events::Updated(event) => {
                    Self::update_user(&mut transaction, user_id, &event.name).await?;
                }
            }
        }

        transaction.commit().await?;
        Ok(())
    }

    async fn delete_lobby<'c, E>(
        executor: E,
        lobby_id: lobby_id::LobbyId,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"DELETE FROM lobby
            WHERE public_id = $1;"#,
            lobby_id.0,
        )
        .execute(executor)
        .await
        .map(|_| ())
    }

    async fn delete_lobby_member<'c, E>(
        executor: E,
        lobby_id: lobby_id::LobbyId,
        user_id: user_id::UserId,
    ) -> Result<bool, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"DELETE
            FROM lobby_member
            WHERE lobby_id = ((SELECT id FROM lobby WHERE public_id = $1))
                AND user_id = ((SELECT id FROM "user" WHERE public_id = $2));"#,
            lobby_id.0,
            user_id.0
        )
        .execute(executor)
        .await
        .map(|result| result.rows_affected() > 0)
    }

    async fn insert_lobby<'c, E>(
        executor: E,
        id: lobby_id::LobbyId,
        name: &str,
        passcode: &Option<String>,
        require_passcode: bool,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO lobby (public_id, name, passcode, require_passcode)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (public_id) DO UPDATE
                SET name = $2,
                    passcode = $3,
                    require_passcode = $4;"#,
            id.0,
            name,
            passcode.clone(),
            require_passcode,
        )
        .execute(executor)
        .await
        .map(|_| ())
    }

    async fn insert_lobby_member<'c, E>(
        executor: E,
        lobby_id: lobby_id::LobbyId,
        user_id: user_id::UserId,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO lobby_member (lobby_id, user_id, host)
            VALUES ((SELECT id FROM lobby WHERE public_id = $1),
                    (SELECT id FROM "user" WHERE public_id = $2),
                    $3)
            ON CONFLICT DO NOTHING;"#,
            lobby_id.0,
            user_id.0,
            false
        )
        .execute(executor)
        .await
        .map(|_| ())
    }

    async fn insert_local<'c, E>(
        conn: E,
        local_id: local_id::LocalId,
        user_id: user_id::UserId,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO local (public_id, user_id)
            VALUES ($1,
                    (SELECT id FROM "user" WHERE "user".public_id = $2));"#,
            local_id.0,
            user_id.0
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    async fn insert_user<'c, E>(conn: E, id: user_id::UserId, name: &str) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO "user" (public_id, name)
            VALUES ($1, $2);"#,
            id.0,
            name,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    async fn notify_lobby<'c, E>(
        conn: E,
        lobby_id: lobby_id::LobbyId,
        lobby_request: LobbyRequest,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let frame = LobbyFrame::new_request(None, lobby_request)
            .to_string()
            .unwrap();

        sqlx::query!(
            r#"SELECT pg_notify($1, $2)"#,
            format!("/lobbies/{}", lobby_id.0),
            frame
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    async fn update_lobby<'c, E>(
        executor: E,
        lobby_id: lobby_id::LobbyId,
        name: &str,
        passcode: &Option<String>,
        require_passcode: bool,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"UPDATE lobby
            SET name = $2,
                passcode = $3,
                require_passcode = $4
            WHERE public_id = $1"#,
            lobby_id.0,
            name,
            passcode.clone(),
            require_passcode,
        )
        .execute(executor)
        .await
        .map(|_| ())
    }

    async fn update_lobby_member_host<'c, E>(
        executor: E,
        lobby_id: lobby_id::LobbyId,
        user_id: user_id::UserId,
        host: bool,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"UPDATE lobby_member
            SET host = $3
            WHERE lobby_id = (SELECT id FROM lobby WHERE public_id = $1)
              AND user_id = (SELECT id FROM "user" WHERE public_id = $2);"#,
            lobby_id.0,
            user_id.0,
            host
        )
        .execute(executor)
        .await
        .map(|_| ())
    }

    async fn update_user<'c, E>(
        conn: E,
        user_id: user_id::UserId,
        name: &str,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"UPDATE "user"
            SET name = $2
            WHERE public_id = $1"#,
            user_id.0,
            name,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeysetPagination {
    pub id: i64,

    pub limit: i64,
}

impl TryFrom<jsonapi::Pagination> for KeysetPagination {
    type Error = jsonapi::Error;

    fn try_from(value: jsonapi::Pagination) -> Result<Self, Self::Error> {
        let parser = |default, value: Option<String>, name: &str| {
            value
                .map_or(Ok(default), |s| s.parse::<i64>())
                .map_err(|error| jsonapi::Error {
                    status: 400,
                    source: Source {
                        header: None,
                        parameter: name.to_string().into(),
                        pointer: None,
                    }
                    .into(),
                    title: "Invalid Query Parameter".to_string().into(),
                    detail: error.to_string().into(),
                })
        };

        let after = parser(0, value.after, "page[after]")?;
        let size = parser(10, value.size, "page[size]")?;

        Ok(KeysetPagination {
            id: after,
            limit: size,
        })
    }
}
