#![deny(clippy::pedantic)]

use futures::{
    channel::mpsc::{channel, Sender},
    SinkExt, StreamExt,
};
use gloo_console::{error, log};
use gloo_net::{
    http::Request,
    websocket::{futures::WebSocket, Message},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    html! {
        <div>
            { "App" }
            <ApiService />
            <WebsocketService />
        </div>
    }
}

struct ApiService {}

impl Component for ApiService {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        spawn_local(async move {
            let response = Request::get("http://localhost:3000/api/v1/ping")
                .send()
                .await
                .unwrap();
        });

        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {}
    }
}

struct WebsocketService {
    _tx: Sender<String>,
}

impl Component for WebsocketService {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let (tx_in, mut rx_in) = channel::<String>(1000);

        let ws = WebSocket::open("ws://localhost:3000/api/v1/ws").unwrap();
        let (mut sink, mut stream) = ws.split();

        spawn_local(async move {
            while let Some(msg) = rx_in.next().await {
                if let Err(err) = sink.send(Message::Text(msg)).await {
                    error!(format!("{err:?}"));
                }
            }
        });

        spawn_local(async move {
            while let Some(msg) = stream.next().await {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!(format!("{err:?}"));
                        return;
                    }
                };

                log!(format!("{msg:?}"));
            }
        });

        Self { _tx: tx_in }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {}
    }
}
