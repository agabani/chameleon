#![deny(clippy::pedantic)]

mod args;
mod database;
mod domain;
mod error;
mod extract;
mod routes;

use std::net::SocketAddr;

use args::Args;
use axum::{
    routing::{get, post},
    Router,
};
use axum_extra::routing::SpaRouter;
use routes::{
    api_v1_lobbies, api_v1_message, api_v1_ping, api_v1_telemetry, api_v1_userinfo, api_v1_users,
    ws_v1,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

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
        .nest(api_v1_users::PATH, api_v1_users::router())
        .nest(api_v1_userinfo::PATH, api_v1_userinfo::router())
        .route("/api/v1/message", post(api_v1_message::post))
        .route("/api/v1/ping", get(api_v1_ping::get))
        .route("/api/v1/telemetry", post(api_v1_telemetry::post))
        .route("/ws/v1", get(ws_v1::get))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    postgres_pool: Pool<Postgres>,
}
