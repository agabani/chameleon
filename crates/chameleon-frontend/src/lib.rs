#![deny(clippy::pedantic)]

mod components;
mod services;

use std::rc::Rc;

use yew::prelude::*;

use crate::{components::chat::Chat, services::Service};

#[function_component]
pub fn App() -> Html {
    let service = use_memo(|_| Service::default(), ());

    html! {
        <ContextProvider<Rc<Service>> context={service}>
            <Chat />
        </ContextProvider<Rc<Service>>>
    }
}
