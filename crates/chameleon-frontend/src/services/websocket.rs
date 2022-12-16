use futures::{
    channel::mpsc::{channel, Sender},
    Future, FutureExt, SinkExt, StreamExt,
};
use gloo::{
    console::error,
    net::websocket::{futures::WebSocket, Message},
    utils::document,
};
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct WebSocketService {}

impl WebSocketService {
    #[allow(clippy::unused_self)]
    pub fn subscribe<S, C, Fut>(&self, state: S, callback: C) -> Sender<String>
    where
        C: Fn(S, Message) -> Fut + 'static,
        S: Clone + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let location = document().location().expect("Failed to read location");
        let protocol = location
            .protocol()
            .expect("Failed to get location protocol");
        let host = location.host().expect("Failed to get location host");

        let url = format!(
            "{}//{}/ws/v1",
            if protocol == "https:" { "wss:" } else { "ws:" },
            host
        );

        let ws = WebSocket::open(&url).unwrap();

        let (tx_send, mut tx_recv) = channel::<String>(1000);

        spawn_local(async move {
            let (mut sink, mut stream) = ws.split();

            while futures::select! {
                write = tx_recv.next() => {
                    match write {
                        Some(msg) => {
                            if let Err(err) = sink.send(Message::Text(msg)).await {
                                error!(format!("{err:?}"));
                                false
                            } else {
                                true
                            }
                        },
                        None => false
                    }
                },
                read = stream.next().fuse() => {
                    match read {
                        Some(msg) => {
                            match msg {
                                Ok(msg) => {
                                    callback(state.clone(), msg).await;
                                    true
                                },
                                Err(err) => {
                                    error!(format!("{err:?}"));
                                    false
                                }
                            }
                        }
                        None => false
                    }
                }
            } {}

            let ws = sink.reunite(stream).expect("Failed to reunite streams");

            ws.close(None, None).expect("Failed to close websocket");
        });

        tx_send
    }
}
