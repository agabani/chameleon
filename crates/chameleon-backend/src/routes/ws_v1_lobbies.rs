use std::str::FromStr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use chameleon_protocol::{
    frames::{LobbyFrame, LobbyRequest, LobbyResponse},
    jsonapi,
    jsonrpc::FrameType,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tracing::Instrument;

use crate::{
    app::AppState,
    database::Database,
    domain::{lobby, lobby_id, local_id, user_id},
    error::ApiError,
};

pub const PATH: &str = "/ws/v1/lobbies";

pub fn router() -> Router<AppState> {
    Router::new().route("/:id", get(get_one))
}

#[tracing::instrument(skip(app_state, web_socket_upgrade))]
async fn get_one(
    State(app_state): State<AppState>,
    Path(lobby_id): Path<lobby_id::LobbyId>,
    web_socket_upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&app_state.pool, lobby_id)
        .await?
        .ok_or_else(|| jsonapi::Error::not_found("lobby", "Lobby"))?;

    Ok(web_socket_upgrade
        .on_upgrade(move |web_socket| get_one_handler(app_state, lobby, web_socket)))
}

#[tracing::instrument(skip(app_state, lobby, web_socket))]
async fn get_one_handler(app_state: AppState, lobby: lobby::Lobby, web_socket: WebSocket) {
    let (mut sink, mut stream) = web_socket.split();

    let mut listener = Database::listener(&app_state.pool)
        .await
        .expect("TODO: Failed to get listener");

    let user_id = match authentication(&app_state, &mut stream, &mut sink).await {
        Ok(Some(user_id)) => user_id,
        Ok(None) => {
            return;
        }
        Err(error) => {
            tracing::error!(error =? error, "error");
            return;
        }
    };

    tracing::info!(user_id =? user_id, "authenticated");

    if !lobby.is_member(user_id) {
        sink.send(Message::Text(
            LobbyFrame::new_error(None, 403, "Forbidden".to_string())
                .to_string()
                .unwrap(),
        ))
        .await
        .unwrap();
        return;
    }

    tokio::spawn(
        async move {
            listener
                .listen(&format!("/lobbies/{}", lobby.id.0))
                .await
                .expect("TODO: failed to listen");

            let mut notify_stream = listener.into_stream();

            loop {
                tokio::select! {
                    message = notify_stream.next() => {
                        let Some(Ok(message)) = message else {
                            // stream closed
                            return;
                        };

                        let Ok(frame) = LobbyFrame::try_from_str(message.payload()) else {
                            // malformed frame
                            return;
                        };

                        sink.send(Message::Text(
                            frame.to_string().unwrap(),
                        ))
                        .await
                        .unwrap();
                    }
                    message = stream.next() => {
                        let Some(Ok(message)) = message else {
                            // stream closed
                            return;
                        };

                        match message {
                            Message::Text(text) => {
                                match LobbyFrame::try_from_str(&text) {
                                    Ok(frame) => {
                                        tracing::info!(frame =? frame, "frame received");
                                    }
                                    Err(_) => {
                                        sink.send(Message::Text(
                                            LobbyFrame::parse_error().to_string().unwrap(),
                                        ))
                                        .await
                                        .expect("TODO: Failed to resend error");
                                    }
                                };
                            }
                            Message::Close(reason) => tracing::info!(reason =? reason, "close"),
                            _ => {}
                        }
                    }
                };
            }
        }
        .in_current_span(),
    );
}

async fn authentication(
    app_state: &AppState,
    stream: &mut SplitStream<WebSocket>,
    sink: &mut SplitSink<WebSocket, Message>,
) -> Result<Option<user_id::UserId>, axum::Error> {
    while let Some(message) = stream.next().await {
        let Message::Text(text) = message? else {
            continue;
        };

        let Ok(frame) = LobbyFrame::try_from_str(&text) else {
            sink.send(Message::Text(LobbyFrame::parse_error().to_string().unwrap())).await.unwrap();
            continue;
        };

        let FrameType::Request(request) = frame.type_ else {
            continue;
        };

        let id = request.id;

        let LobbyRequest::Authenticate(request) = request.data else {
            continue;
        };

        let Some(local_id) = request.local_id else {
            sink.send(Message::Text(LobbyFrame::parse_error().to_string().unwrap())).await.unwrap();
            continue;
        };

        let Ok(local_id) = local_id::LocalId::from_str(&local_id) else {
            sink.send(Message::Text(LobbyFrame::parse_error().to_string().unwrap())).await.unwrap();
            continue;
        };

        let Ok(user_id) =
            Database::select_user_id_by_local_id(&app_state.pool, local_id).await else {
                sink.send(Message::Text(LobbyFrame::internal_error(id).to_string().unwrap())).await.unwrap();
                continue;
            };

        let Some(user_id) = user_id else {
            sink.send(Message::Text(LobbyFrame::new_response(id, LobbyResponse::Authenticate(false)).to_string().unwrap())).await.unwrap();
            continue;
        };

        sink.send(Message::Text(
            LobbyFrame::new_response(id, LobbyResponse::Authenticate(true))
                .to_string()
                .unwrap(),
        ))
        .await
        .unwrap();
        return Ok(Some(user_id));
    }

    Ok(None)
}
