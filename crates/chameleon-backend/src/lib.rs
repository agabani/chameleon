#![deny(clippy::pedantic)]

use std::time::Duration;

use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

#[allow(clippy::missing_panics_doc)]
pub async fn app() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/v1/ping", get(api_v1_ping))
        .route("/api/v1/ws", get(api_v1_websocket))
        .layer(CorsLayer::new().allow_origin(["http://localhost:8080".parse().unwrap()]));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn api_v1_ping() -> Json<Value> {
    Json(json!({}))
}

async fn api_v1_websocket(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(v1_websocket_callback)
}

async fn v1_websocket_callback(websocket: WebSocket) {
    let (mut sink, mut stream) = websocket.split();

    tokio::spawn(async move {
        let period = Duration::from_secs(1);

        loop {
            if let Err(err) = sink.send(axum::extract::ws::Message::Ping(vec![])).await {
                tracing::info!(err =? err, "client disconnected");
                return;
            }

            tokio::time::sleep(period).await;
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
