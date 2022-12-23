use chameleon_protocol::{
    jsonapi::{self, Source},
    ws,
};
use sqlx::{postgres::PgListener, Executor, Pool, Postgres};

use crate::domain::{Game, GameId, LocalId, User, UserId};

pub struct Database {}

impl Database {
    pub async fn get_user_id_by_local_id<'c, E>(
        local_id: LocalId,
        conn: E,
    ) -> Result<Option<UserId>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT u.public_id
            FROM "user" AS u
                     JOIN local AS l ON u.id = l.user_id
            WHERE l.public_id = $1;"#,
            local_id.value(),
        )
        .map(|record| UserId::new(record.public_id))
        .fetch_optional(conn)
        .await
    }

    pub async fn save_local_id<'c, E>(
        user_id: UserId,
        local_id: LocalId,
        conn: E,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO local (public_id, user_id)
            VALUES ($2,
                    (SELECT u.id FROM "user" u WHERE u.public_id = $1))
            ON CONFLICT DO NOTHING;"#,
            user_id.value(),
            local_id.value(),
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn get_user_by_id<'c, E>(id: UserId, conn: E) -> Result<Option<User>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT name
            FROM "user"
            WHERE public_id = $1"#,
            id.value()
        )
        .fetch_optional(conn)
        .await
        .map(|record| record.map(|record| User::new(id, record.name)))
    }

    pub async fn save_user<'c, E>(user: &User, conn: E) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO "user" (public_id, name)
            VALUES ($1, $2)
            ON CONFLICT (public_id) DO UPDATE
                SET name = $2;"#,
            user.id().value(),
            user.name()
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn notify<'c, E>(
        conn: E,
        chan: &str,
        response: &ws::Response,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(r#"SELECT pg_notify($1, $2)"#, chan, response.to_string())
            .execute(conn)
            .await
            .map(|_| ())
    }

    pub async fn listener(conn: &Pool<Postgres>) -> Result<PgListener, sqlx::Error> {
        PgListener::connect_with(conn).await
    }

    pub async fn insert_game<'c, E>(conn: E, game: &Game) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO game (public_id, name)
            VALUES ($1, $2)
            ON CONFLICT (public_id) DO UPDATE
                SET name = $2;"#,
            game.id.0,
            game.name,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn select_game<'c, E>(conn: E, game_id: GameId) -> Result<Option<Game>, sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"SELECT g.public_id, g.name, u.public_id host_public_id
            FROM game g
                     JOIN game_player gp ON g.id = gp.game_id
                     JOIN "user" u ON u.id = gp.user_id
            WHERE g.public_id = $1
              AND gp.host IS TRUE;"#,
            game_id.0
        )
        .fetch_optional(conn)
        .await
        .map(|record| {
            record.map(|record| Game {
                id: GameId(record.public_id),
                name: record.name,
                host: UserId::new(record.host_public_id),
            })
        })
    }

    pub async fn query_game<'c, E>(
        conn: E,
        keyset_pagination: KeysetPagination,
    ) -> Result<(Vec<Game>, i64), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let records = sqlx::query!(
            r#"SELECT g.id, g.public_id, g.name, u.public_id host_public_id
            FROM game g
                     JOIN game_player gp ON g.id = gp.game_id
                     JOIN "user" u ON u.id = gp.user_id
            WHERE g.id > $1
              AND gp.host IS TRUE
            ORDER BY g.id
            LIMIT $2;"#,
            keyset_pagination.id,
            keyset_pagination.limit,
        )
        .fetch_all(conn)
        .await?;

        let last_record_id = records
            .last()
            .map_or(keyset_pagination.id, |record| record.id);

        let games = records
            .into_iter()
            .map(|record| Game {
                id: GameId(record.public_id),
                name: record.name,
                host: UserId::new(record.host_public_id),
            })
            .collect();

        Ok((games, last_record_id))
    }

    pub async fn update_game<'c, E>(conn: E, game: &Game) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"UPDATE game
            SET name = $2
            WHERE public_id = $1"#,
            game.id.0,
            game.name,
        )
        .execute(conn)
        .await
        .map(|_| ())
    }

    pub async fn insert_game_player<'c, E>(conn: E, game: &Game) -> Result<(), sqlx::Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        sqlx::query!(
            r#"INSERT INTO game_player (game_id, user_id, host)
            VALUES ((SELECT id FROM game WHERE public_id = $1),
                    (SELECT id FROM "user" WHERE public_id = $2),
                    $3);"#,
            game.id.0,
            game.host.0,
            true
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
