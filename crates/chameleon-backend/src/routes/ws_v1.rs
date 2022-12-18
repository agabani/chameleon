use std::{net::SocketAddr, str::FromStr, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::Response,
};
use chameleon_protocol::ws;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::time::sleep;

use crate::{database::Database, domain::LocalId, AppState};

#[allow(clippy::unused_async)]
#[tracing::instrument(skip(state, ws))]
pub async fn get(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    tracing::info!("request");
    ws.on_upgrade(|ws| ws_v1_handler(ws, state))
}

async fn ws_v1_handler(websocket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = websocket.split();

    let mut listener = Database::listener(&state.postgres_pool)
        .await
        .expect("Failed to get listener");

    let local_id = match authenticate(&mut stream, &mut sink).await {
        Ok(Some(result)) => result,
        Ok(None) => {
            tracing::warn!("stream disconnected");
            return;
        }
        Err(err) => {
            tracing::error!(err =? err,"stream disconnected");
            return;
        }
    };

    tracing::info!(
        local_id =? local_id,
    "client authenticated");

    tokio::spawn(async move {
        let period = Duration::from_secs(1);

        listener
            .listen("testing")
            .await
            .expect("Failed to subscribe");

        let mut notify_stream = listener.into_stream();

        loop {
            tokio::select! {
                msg = notify_stream.next() => {
                    let Some(msg) = msg else {
                        tracing::info!("stream disconnected");
                        return;
                    };

                    let msg = msg.expect("");

                    let payload: String = msg.payload().to_string();
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

            match msg {
                Message::Text(_) | Message::Binary(_) | Message::Close(_) => {
                    tracing::info!(msg =? msg, "msg");
                }
                Message::Ping(_) | Message::Pong(_) => {}
            };
        }
    });
}

async fn authenticate(
    stream: &mut SplitStream<WebSocket>,
    sink: &mut SplitSink<WebSocket, Message>,
) -> Result<Option<LocalId>, axum::Error> {
    while let Some(msg) = stream.next().await {
        let msg = msg?;

        match msg {
            axum::extract::ws::Message::Text(json) => match serde_json::from_str(&json).unwrap() {
                ws::Request::Authenticate(request) => {
                    let local_id = LocalId::from_str(&request.local_id).unwrap();

                    sink.send(Message::Text(ws::Response::Authenticated.to_string()))
                        .await
                        .unwrap();

                    return Ok(Some(local_id));
                }
            },
            axum::extract::ws::Message::Binary(_)
            | axum::extract::ws::Message::Ping(_)
            | axum::extract::ws::Message::Pong(_) => {
                // Protocol expects first message is authentication, ignore everything else.
            }
            axum::extract::ws::Message::Close(_) => {
                return Ok(None);
            }
        }
    }

    todo!()
}
