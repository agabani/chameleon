#![deny(clippy::pedantic)]

mod domain;
mod error;
mod extract;
mod routes;

use axum::{
    routing::{get, post, put},
    Router,
};
use axum_extra::routing::SpaRouter;
use routes::{api_v1_message, api_v1_ping, api_v1_user, api_v1_users, ws_v1};

#[allow(clippy::missing_panics_doc)]
pub async fn app() {
    let redis_client =
        redis::Client::open("redis://localhost:6379").expect("Failed to create redis client");
    let redis_connection = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to create redis connection");

    let state = AppState {
        redis_client,
        redis_connection,
    };

    let app = Router::new()
        .merge(SpaRouter::new("/", "dist"))
        .route("/api/v1/message", post(api_v1_message::post))
        .route("/api/v1/ping", get(api_v1_ping::get))
        .route("/api/v1/user", get(api_v1_user::get))
        .route("/api/v1/user", put(api_v1_user::put))
        .route("/api/v1/users/:user_id", get(api_v1_users::get))
        .route("/ws/v1", get(ws_v1::get))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    redis_client: redis::Client,
    redis_connection: redis::aio::MultiplexedConnection,
}
