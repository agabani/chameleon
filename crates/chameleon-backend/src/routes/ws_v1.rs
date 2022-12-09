use std::{str::FromStr, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use chameleon_protocol::ws;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::time::sleep;

use crate::{
    domain::{LocalId, SessionId},
    AppState,
};

#[allow(clippy::unused_async)]
#[tracing::instrument(skip(state, ws))]
pub async fn get(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    tracing::info!("request");
    ws.on_upgrade(|ws| ws_v1_handler(ws, state))
}

async fn ws_v1_handler(websocket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = websocket.split();

    let connection = state
        .redis_client
        .get_async_connection()
        .await
        .expect("Failed to get connection");

    let (local_id, session_id) = match authenticate(&mut stream, &mut sink).await {
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
        session_id =? session_id,
    "client authenticated");

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

async fn authenticate(
    stream: &mut SplitStream<WebSocket>,
    sink: &mut SplitSink<WebSocket, Message>,
) -> Result<Option<(LocalId, SessionId)>, axum::Error> {
    while let Some(msg) = stream.next().await {
        let msg = msg?;

        match msg {
            axum::extract::ws::Message::Text(json) => match serde_json::from_str(&json).unwrap() {
                ws::Request::Authenticate(request) => {
                    let local_id = LocalId::from_str(&request.local_id).unwrap();
                    let session_id = SessionId::from_str(&request.session_id).unwrap();

                    sink.send(Message::Text(ws::Response::Authenticated.to_string()))
                        .await
                        .unwrap();

                    return Ok(Some((local_id, session_id)));
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
