pub mod api;
pub mod websocket;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Service {
    pub api: api::ApiService,
    pub websocket: websocket::WebSocketService,
}
