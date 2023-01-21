use chameleon_protocol::{
    frames::{LobbyFrame, LobbyRequest},
    jsonapi::{self, Source},
};
use sqlx::{postgres::PgListener, Executor, Pool, Postgres};

use crate::{
    domain::{LobbyId, LocalId, UserId},
    domain_old::{Lobby, User},
};

pub struct Database {}

impl Database {
    pub async fn listener(conn: &Pool<Postgres>) -> Result<PgListener, sqlx::Error> {
        PgListener::connect_with(conn).await
    }

    pub async fn notify_lobby<'c, E>(
        conn: E,
        lobby_id: LobbyId,
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

    pub async fn delete_lobby_member<'c, E>(
        conn: E,
        lobby: &Lobby,
        user: &User,
    ) -> Result<bool, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"DELETE
            FROM lobby_member
            WHERE lobby_id = ((SELECT id FROM lobby WHERE public_id = $1))
                AND user_id = ((SELECT id FROM "user" WHERE public_id = $2));"#,
            lobby.id.0,
            user.id.0
        )
        .execute(conn)
        .await
        .map(|result| result.rows_affected() > 0)
    }

    pub async fn insert_lobby<'c, E>(conn: E, lobby: &Lobby) -> Result<(), sqlx::Error>
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
            lobby.id.0,
            lobby.name,
            lobby.passcode,
            lobby.require_passcode,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn insert_lobby_host<'c, E>(conn: E, lobby: &Lobby) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO lobby_member (lobby_id, user_id, host)
            VALUES ((SELECT id FROM lobby WHERE public_id = $1),
                    (SELECT id FROM "user" WHERE public_id = $2),
                    $3);"#,
            lobby.id.0,
            lobby.host.0,
            true
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn insert_lobby_member<'c, E>(
        conn: E,
        lobby: &Lobby,
        user: &User,
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
            lobby.id.0,
            user.id.0,
            false
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn insert_local<'c, E>(
        conn: E,
        local_id: LocalId,
        user_id: UserId,
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

    pub async fn insert_user<'c, E>(conn: E, user: &User) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO "user" (public_id, name)
            VALUES ($1, $2);"#,
            user.id.0,
            user.name,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn is_lobby_member<'c, E>(
        conn: E,
        lobby_id: LobbyId,
        user_id: UserId,
    ) -> Result<bool, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT TRUE member
            FROM lobby l
                     JOIN lobby_member lm on l.id = lm.lobby_id
                     JOIN "user" u on u.id = lm.user_id
            WHERE l.public_id = $1
              AND u.public_id = $2;"#,
            lobby_id.0,
            user_id.0
        )
        .fetch_optional(conn)
        .await
        .map(|r| r.and_then(|r| r.member).unwrap_or_default())
    }

    pub async fn query_lobby<'c, E>(
        conn: E,
        keyset_pagination: KeysetPagination,
    ) -> Result<(Vec<Lobby>, i64), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let records = sqlx::query!(
            r#"SELECT l.id, l.public_id, l.name, l.passcode, l.require_passcode, u.public_id host_public_id
            FROM lobby l
                     JOIN lobby_member lm ON l.id = lm.lobby_id
                     JOIN "user" u ON u.id = lm.user_id
            WHERE l.id > $1
              AND lm.host IS TRUE
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
            .map(|record| Lobby {
                id: LobbyId(record.public_id),
                name: record.name,
                host: UserId(record.host_public_id),
                passcode: record.passcode,
                require_passcode: record.require_passcode,
            })
            .collect();

        Ok((lobbies, last_record_id))
    }

    pub async fn query_lobby_member<'c, E>(
        conn: E,
        lobby_id: LobbyId,
        keyset_pagination: KeysetPagination,
    ) -> Result<(Vec<User>, i64), sqlx::Error>
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
            .map(|record| User {
                id: UserId(record.public_id),
                name: record.name,
            })
            .collect();

        Ok((users, last_record_id))
    }

    pub async fn select_lobby<'c, E>(
        conn: E,
        lobby_id: LobbyId,
    ) -> Result<Option<Lobby>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT l.public_id, l.name, l.passcode, l.require_passcode, u.public_id host_public_id
            FROM lobby l
                     JOIN lobby_member lm ON l.id = lm.lobby_id
                     JOIN "user" u ON u.id = lm.user_id
            WHERE l.public_id = $1
              AND lm.host IS TRUE;"#,
            lobby_id.0
        )
        .fetch_optional(conn)
        .await
        .map(|record| {
            record.map(|record| Lobby {
                id: LobbyId(record.public_id),
                name: record.name,
                host: UserId(record.host_public_id),
                passcode: record.passcode,
                require_passcode: record.require_passcode
            })
        })
    }

    pub async fn select_user<'c, E>(conn: E, user_id: UserId) -> Result<Option<User>, sqlx::Error>
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
            record.map(|record| User {
                id: UserId(record.public_id),
                name: record.name,
            })
        })
    }

    pub async fn select_user_id_by_local_id<'c, E>(
        conn: E,
        local_id: LocalId,
    ) -> Result<Option<UserId>, sqlx::Error>
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
        .map(|record| UserId(record.public_id))
        .fetch_optional(conn)
        .await
    }

    pub async fn update_lobby<'c, E>(conn: E, lobby: &Lobby) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"UPDATE lobby
            SET name = $2,
                passcode = $3,
                require_passcode = $4
            WHERE public_id = $1"#,
            lobby.id.0,
            lobby.name,
            lobby.passcode,
            lobby.require_passcode,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn update_user<'c, E>(conn: E, user: &User) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"UPDATE "user"
            SET name = $2
            WHERE public_id = $1"#,
            user.id.0,
            user.name,
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
