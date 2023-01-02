use std::net::SocketAddr;

use axum::Router;
use axum_extra::routing::SpaRouter;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{
    args::Args,
    routes::{api_v1_lobbies, api_v1_ping, api_v1_userinfo, api_v1_users, ws_v1_lobbies},
};

#[allow(clippy::missing_panics_doc)]
pub async fn app() {
    let args = Args::parse();

    let postgres_pool = PgPoolOptions::new()
        .connect(&args.postgres_url)
        .await
        .expect("Failed to create postgres connection");

    sqlx::migrate!()
        .run(&postgres_pool)
        .await
        .expect("Failed to migrate postgres database");

    let state = AppState { postgres_pool };

    let app = Router::new()
        .merge(SpaRouter::new("/assets", "dist"))
        .nest(api_v1_lobbies::PATH, api_v1_lobbies::router())
        .nest(api_v1_ping::PATH, api_v1_ping::router())
        .nest(api_v1_users::PATH, api_v1_users::router())
        .nest(api_v1_userinfo::PATH, api_v1_userinfo::router())
        .nest(ws_v1_lobbies::PATH, ws_v1_lobbies::router())
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: Pool<Postgres>,
}
