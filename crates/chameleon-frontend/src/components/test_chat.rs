use std::{collections::HashMap, rc::Rc, str::FromStr};

use chameleon_protocol::{http, ws};
use futures::channel::mpsc::Sender;
use web_sys::HtmlInputElement;
use yew::{html::Scope, prelude::*};

use crate::services::{
    storage::{local_id, session_id},
    Service,
};

#[derive(Debug, Clone)]
pub struct TestChat {
    connection_status: ConnectionStatus,
    input: NodeRef,
    messages: Vec<ws::MessageResponse>,
    user_name: HashMap<String, String>,
    websocket_sender: Option<Sender<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Disconnecting,
}

impl ConnectionStatus {
    fn as_input(self) -> &'static str {
        match self {
            ConnectionStatus::Connected | ConnectionStatus::Disconnecting => "disconnect",
            ConnectionStatus::Disconnected | ConnectionStatus::Connecting => "connect",
        }
    }

    fn as_status(self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "connected",
            ConnectionStatus::Connecting => "connecting",
            ConnectionStatus::Disconnected => "disconnected",
            ConnectionStatus::Disconnecting => "disconnecting",
        }
    }

    fn is_transitioning(self) -> bool {
        match self {
            ConnectionStatus::Connected | ConnectionStatus::Disconnected => false,
            ConnectionStatus::Connecting | ConnectionStatus::Disconnecting => true,
        }
    }

    fn is_connected(self) -> bool {
        self == ConnectionStatus::Connected
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    ConnectClicked,
    SenderCreated(Sender<String>),
    DisconnectClicked,
    WsResponseRecieved(ws::Response),
    SubmitClicked,
    MessageSent,
    Connected,
    SenderDropped,
}

impl Component for TestChat {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            connection_status: ConnectionStatus::Disconnected,
            input: NodeRef::default(),
            messages: Vec::new(),
            user_name: HashMap::new(),
            websocket_sender: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let connection_button_onclick = ctx.link().callback(match self.connection_status {
            ConnectionStatus::Connected | ConnectionStatus::Connecting => {
                |_| Msg::DisconnectClicked
            }
            ConnectionStatus::Disconnected | ConnectionStatus::Disconnecting => {
                |_| Msg::ConnectClicked
            }
        });
        let form_onsubmit = ctx.link().callback(|event: SubmitEvent| {
            event.prevent_default();
            Msg::SubmitClicked
        });

        html! {
            <div>
                <h2>{ "Test Chat" }</h2>
                <div>{ "connection status: " }{ self.connection_status.as_status() }</div>
                <button
                    disabled={ self.connection_status.is_transitioning() }
                    onclick={ connection_button_onclick }>
                    { self.connection_status.as_input() }
                </button>
                <ul>
                    { self.messages.iter().map(|message| html! {
                        <li>{ &self.user_name[&message.user_id] }{ ": " }{ &message.content }</li>
                    }).collect::<Html>() }
                </ul>
                <form onsubmit={ form_onsubmit }>
                    <input
                        disabled={ !self.connection_status.is_connected() }
                        ref={ &self.input }
                        type="text" />
                    <button
                        disabled={ !self.connection_status.is_connected() }
                        type="submit">{ "Send" }</button>
                </form>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ConnectClicked => {
                self.connection_status = ConnectionStatus::Connecting;

                let scope = ctx.link().clone();
                ctx.link().send_message(connect(scope));

                true
            }
            Msg::DisconnectClicked => {
                self.connection_status = ConnectionStatus::Disconnecting;

                drop(self.websocket_sender.take());

                ctx.link().send_message(Msg::SenderDropped);

                false
            }
            Msg::SenderCreated(sender) => {
                self.websocket_sender = Some(sender);
                false
            }
            Msg::WsResponseRecieved(response) => match response {
                ws::Response::Authenticated => {
                    ctx.link().send_message(Msg::Connected);
                    false
                }
                ws::Response::Message(message) => {
                    self.user_name
                        .insert(message.user_id.clone(), message.user_name.clone());
                    self.messages.push(message);
                    true
                }
            },
            Msg::SubmitClicked => {
                let input = self
                    .input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input");

                let value = input.value();
                input.set_value("");

                let scope = ctx.link().clone();
                ctx.link()
                    .send_future(async move { send_message(scope, value).await });

                true
            }
            Msg::MessageSent => false,
            Msg::Connected => {
                self.connection_status = ConnectionStatus::Connected;
                true
            }
            Msg::SenderDropped => {
                self.connection_status = ConnectionStatus::Disconnected;
                true
            }
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {}
}

fn connect(scope: Scope<TestChat>) -> Msg {
    let (service, _) = scope
        .context::<Rc<Service>>(Callback::noop())
        .expect("Failed to find service in context");

    let mut sender = service
        .websocket
        .subscribe(scope, |scope: Scope<TestChat>, msg| async move {
            match msg {
                gloo::net::websocket::Message::Text(text) => {
                    let response = ws::Response::from_str(&text).unwrap();
                    scope.send_message(Msg::WsResponseRecieved(response));
                }
                gloo::net::websocket::Message::Bytes(_) => todo!(),
            };
        });

    sender
        .try_send(
            ws::Request::Authenticate(ws::AuthenticateRequest {
                local_id: session_id().unwrap(),
                session_id: local_id().unwrap(),
            })
            .to_string(),
        )
        .unwrap();

    Msg::SenderCreated(sender)
}

async fn send_message(scope: Scope<TestChat>, message: String) -> Msg {
    let (service, _) = scope
        .context::<Rc<Service>>(Callback::noop())
        .expect("Failed to find service in context");

    let result = service
        .api
        .post_message(&http::MessageRequest { content: message })
        .await;

    match result {
        Ok(_) => Msg::MessageSent,
        Err(_) => todo!(),
    }
}
