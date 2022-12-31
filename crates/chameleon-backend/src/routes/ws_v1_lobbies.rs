use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use chameleon_protocol::{frames::LobbyFrame, jsonapi};
use futures::{SinkExt, StreamExt};
use tracing::Instrument;

use crate::{database::Database, domain::LobbyId, error::ApiError, AppState};

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
