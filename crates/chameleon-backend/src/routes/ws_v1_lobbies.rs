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
    database::Database,
    domain::{LobbyId, LocalId, UserId},
    error::ApiError,
    AppState,
};

pub const PATH: &str = "/ws/v1/lobbies";

pub fn router() -> Router<AppState> {
    Router::new().route("/:id", get(get_one))
}

async fn get_one(
    State(app_state): State<AppState>,
    Path(lobby_id): Path<LobbyId>,
    web_socket_upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let _lobby = Database::select_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| jsonapi::Error::not_found("lobby", "Lobby"))?;

    Ok(web_socket_upgrade
        .on_upgrade(move |web_socket| get_one_handler(app_state, lobby_id, web_socket)))
}

#[tracing::instrument(skip(app_state, web_socket))]
async fn get_one_handler(app_state: AppState, lobby_id: LobbyId, web_socket: WebSocket) {
    let (mut sink, mut stream) = web_socket.split();

    let mut _listener = Database::listener(&app_state.postgres_pool)
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

    match Database::is_lobby_member(&app_state.postgres_pool, lobby_id, user_id).await {
        Ok(true) => {
            // do nothing...
        }
        Ok(false) => {
            sink.send(Message::Text(
                LobbyFrame::new_error(None, 403, "Forbidden".to_string())
                    .to_string()
                    .unwrap(),
            ))
            .await
            .unwrap();
            return;
        }
        Err(error) => {
            sink.send(Message::Text(
                LobbyFrame::internal_error(None).to_string().unwrap(),
            ))
            .await
            .unwrap();
            tracing::error!(error =? error, "error");
            return;
        }
    };

    tokio::spawn(
        async move {
            while let Some(message) = stream.next().await {
                let Ok(message) = message else {
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
        }
        .in_current_span(),
    );
}

async fn authentication(
    app_state: &AppState,
    stream: &mut SplitStream<WebSocket>,
    sink: &mut SplitSink<WebSocket, Message>,
) -> Result<Option<UserId>, axum::Error> {
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

        let LobbyRequest::Authenticate(request) = request.data;

        let Some(local_id) = request.local_id else {
            sink.send(Message::Text(LobbyFrame::parse_error().to_string().unwrap())).await.unwrap();
            continue;
        };

        let Ok(local_id) = LocalId::from_str(&local_id) else {
            sink.send(Message::Text(LobbyFrame::parse_error().to_string().unwrap())).await.unwrap();
            continue;
        };

        let Ok(user_id) =
            Database::select_user_id_by_local_id(&app_state.postgres_pool, local_id).await else {
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
