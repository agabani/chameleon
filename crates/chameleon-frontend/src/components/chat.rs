use std::rc::Rc;

use futures::channel::mpsc::Sender;
use gloo::{console::error, net::websocket::Message};
use web_sys::HtmlInputElement;
use yew::{html::Scope, prelude::*};

use crate::services::Service;

pub enum Msg {
    Submit,
    Submitted,
    MessageReceived(String),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Props {}

pub struct Chat {
    input: NodeRef,
    messages: Vec<String>,
    _sender: Sender<String>,
}

impl Component for Chat {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (service, _) = ctx
            .link()
            .context::<Rc<Service>>(Callback::noop())
            .expect("Failed to find service in context");

        let sender =
            service
                .websocket
                .subscribe(ctx.link().clone(), |link: Scope<Self>, msg| async move {
                    match msg {
                        Message::Text(text) => {
                            link.send_message(Msg::MessageReceived(text));
                        }
                        Message::Bytes(_) => {}
                    };
                });

        Chat {
            input: NodeRef::default(),
            messages: Vec::new(),
            _sender: sender,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|event: SubmitEvent| {
            event.prevent_default();
            Msg::Submit
        });

        html! {
            <div>
                { self.messages.iter().map(|message| { html!{ <div>{ message }</div> } }).collect::<Html>() }
                <form onsubmit={submit}>
                    <input ref={self.input.clone()} type="text" />
                    <button type="submit">{ "Send" }</button>
                </form>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Submit => {
                let value = self
                    .input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input")
                    .value();

                let (service, _) = ctx
                    .link()
                    .context::<Rc<Service>>(Callback::noop())
                    .expect("Failed to find service in context");

                ctx.link().send_future(async move {
                    if let Err(err) = service.api.message(&value).await {
                        error!(format!("Failed to send request {:?}", err));
                    }
                    Msg::Submitted
                });

                false
            }
            Msg::Submitted => {
                self.input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input")
                    .set_value("");

                false
            }
            Msg::MessageReceived(message) => {
                self.messages.push(message);

                true
            }
        }
    }
}
