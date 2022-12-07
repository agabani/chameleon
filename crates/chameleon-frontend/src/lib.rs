#![deny(clippy::pedantic)]

mod components;
mod services;

use std::rc::Rc;

use yew::prelude::*;

use crate::{
    components::{chat::Chat, topic_card::TopicCard},
    services::Service,
};

#[function_component]
pub fn App() -> Html {
    let service = use_memo(|_| Service::default(), ());

    let name = "Sports".to_string();
    let secret_words = vec![
        "Football",
        "Basketball",
        "Tennis",
        "Lacrosse",
        "Soccer",
        "Ice Hockey",
        "Badminton",
        "Volleyball",
        "Golf",
        "Sailing",
        "Motor Racing",
        "Triathlon",
        "Baseball",
        "Squash",
        "Wrestling",
        "Cycling",
    ]
    .into_iter()
    .map(AttrValue::from)
    .collect::<Vec<_>>();

    html! {
        <ContextProvider<Rc<Service>> context={service}>
            <TopicCard {name} {secret_words} />
            <Chat />
        </ContextProvider<Rc<Service>>>
    }
}
