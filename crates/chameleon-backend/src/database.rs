use chameleon_protocol::ws;
use sqlx::{postgres::PgListener, Pool, Postgres};

use crate::domain::{LocalId, User, UserId};

pub struct Database {}

impl Database {
    pub async fn get_user_id_by_local_id(
        local_id: LocalId,
        conn: &Pool<Postgres>,
    ) -> Result<Option<UserId>, sqlx::Error> {
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

    pub async fn save_local_id(
        user_id: UserId,
        local_id: LocalId,
        conn: &Pool<Postgres>,
    ) -> Result<(), sqlx::Error> {
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

    pub async fn get_user_by_id(
        id: UserId,
        conn: &Pool<Postgres>,
    ) -> Result<Option<User>, sqlx::Error> {
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

    pub async fn save_user(user: &User, conn: &Pool<Postgres>) -> Result<(), sqlx::Error> {
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

    pub async fn notify(
        conn: &Pool<Postgres>,
        chan: &str,
        response: &ws::Response,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"SELECT pg_notify($1, $2)"#, chan, response.to_string())
            .execute(conn)
            .await
            .map(|_| ())
    }

    pub async fn listener(conn: &Pool<Postgres>) -> Result<PgListener, sqlx::Error> {
        PgListener::connect_with(conn).await
    }
}
