use std::rc::Rc;

use gloo::console::error;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::NetworkService;

pub enum Msg {
    Submit,
}

#[derive(Properties, PartialEq, Eq)]
pub struct Props {}

pub struct Chat {
    input: NodeRef,
}

impl Component for Chat {
    type Message = Msg;

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Chat {
            input: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|event: SubmitEvent| {
            event.prevent_default();
            Msg::Submit
        });

        html! {
            <div>
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
                let input = self
                    .input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input");

                let (network_service, _handle) = ctx
                    .link()
                    .context::<Rc<NetworkService>>(yew::Callback::noop())
                    .expect("Failed to find network service in context");

                let value = input.value();

                spawn_local(async move {
                    if let Err(err) = network_service.api.message(&value).await {
                        error!(format!("Failed to send request {:?}", err));
                    }
                });

                input.set_value("");
            }
        }

        true
    }
}
