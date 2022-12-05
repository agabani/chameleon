#![deny(clippy::pedantic)]

use std::time::Duration;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::sleep;

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
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/v1/message", post(api_v1_message))
        .route("/api/v1/ping", get(api_v1_ping))
        .route("/ws/v1", get(ws_v1))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
struct AppState {
    redis_client: redis::Client,
    redis_connection: redis::aio::MultiplexedConnection,
}

#[allow(clippy::unused_async)]
#[tracing::instrument]
async fn api_v1_ping() -> Json<Value> {
    tracing::info!("request");
    Json(json!({}))
}

#[tracing::instrument(skip(state))]
async fn api_v1_message(State(mut state): State<AppState>, body: String) -> StatusCode {
    redis::Cmd::publish("testing", body)
        .query_async(&mut state.redis_connection)
        .await
        .map(|_: ()| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

#[allow(clippy::unused_async)]
async fn ws_v1(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    tracing::info!("Incoming connection");
    ws.on_upgrade(|ws| ws_v1_handler(ws, state))
}

async fn ws_v1_handler(websocket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = websocket.split();

    let connection = state
        .redis_client
        .get_async_connection()
        .await
        .expect("Failed to get connection");

    tokio::spawn(async move {
        let period = Duration::from_secs(1);

        let mut pubsub = connection.into_pubsub();
        pubsub
            .subscribe("testing")
            .await
            .expect("Failed to subscribe");

        let mut pubsub_stream = pubsub.on_message();

        loop {
            tokio::select! {
                msg = pubsub_stream.next() => {
                    let Some(msg) = msg else {
                        tracing::info!("stream disconnected");
                        return;
                    };

                    let payload: String = msg.get_payload().expect("Failed to get payload");
                    tracing::info!(payload, "sending payload");

                    if let Err(err) = sink.send(axum::extract::ws::Message::Text(payload)).await {
                        tracing::info!(err =? err, "client disconnected");
                        return;
                    }
                },
                _ = sleep(period) => {
                    if let Err(err) = sink.send(axum::extract::ws::Message::Ping(vec![])).await {
                        tracing::info!(err =? err, "client disconnected");
                        return;
                    }
                }
            };
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = stream.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    tracing::info!(err =? err, "client disconnected");
                    return;
                }
            };

            tracing::info!("{:?}", msg);
        }
    });
}
