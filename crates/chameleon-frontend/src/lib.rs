#![deny(clippy::pedantic)]

use std::rc::Rc;

use futures::{channel::mpsc::channel, SinkExt, StreamExt};
use gloo::{
    console::{error, log},
    net::{
        http::Request,
        websocket::{futures::WebSocket, Message},
    },
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    let network_service = use_memo(|_| NetworkService::default(), ());

    html! {
        <ContextProvider<Rc<NetworkService>> context={network_service}>
            <div>{ "App" }</div>
            <TestApi />
            <TestWs />
        </ContextProvider<Rc<NetworkService>>>
    }
}

#[function_component]
pub fn TestApi() -> Html {
    let network_service = use_context::<Rc<NetworkService>>().expect("no ctx found");

    let onclick = Callback::from(move |_| {
        let network_service = network_service.clone();

        spawn_local(async move {
            let response = network_service.api.ping().await.unwrap();
            log!(format!("{:?}", response));
        });
    });

    html! {
        <div>
            <button {onclick}>{ "API: [GET] /api/v1/ping" }</button>
        </div>
    }
}

#[function_component]
pub fn TestWs() -> Html {
    let network_service = use_context::<Rc<NetworkService>>().expect("no ctx found");

    let onclick = Callback::from(move |_| {
        network_service.ws.subscribe();
        log!("subscribed...");
    });

    html! {
        <div>
            <button {onclick}>{ "WS: [GET] /ws/v1" }</button>
        </div>
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct NetworkService {
    api: ApiService,
    ws: WebsocketService,
}

#[derive(Clone, Default, PartialEq, Eq)]
struct ApiService {}

impl ApiService {
    async fn ping(&self) -> Result<gloo::net::http::Response, gloo::net::Error> {
        Request::get("/api/v1/ping").send().await
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
struct WebsocketService {}

impl WebsocketService {
    fn subscribe(&self) {
        let (_tx_in, mut rx_in) = channel::<String>(1000);

        let url = gloo::utils::document()
            .location()
            .map(|location| {
                format!(
                    "{}://{}/ws/v1",
                    if location.protocol().unwrap() == "https" {
                        "wss"
                    } else {
                        "ws"
                    },
                    location.host().unwrap()
                )
            })
            .unwrap();

        let ws = WebSocket::open(&url).unwrap();
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
    }
}
