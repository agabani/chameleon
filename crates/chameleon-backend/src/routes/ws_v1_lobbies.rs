use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use chameleon_protocol::jsonapi;
use futures::StreamExt;
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
    let (mut _sink, mut stream) = web_socket.split();

    let mut _listener = Database::listener(&app_state.postgres_pool)
        .await
        .expect("TODO: Failed to get listener");

    tokio::spawn(
        async move {
            while let Some(message) = stream.next().await {
                let Ok(message) = message else {
                return;
            };

                tracing::info!(message =? message, "message received");
            }
        }
        .in_current_span(),
    );
}
