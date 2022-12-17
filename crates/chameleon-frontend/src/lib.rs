#![deny(clippy::pedantic)]

mod components;
mod services;

use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::{test_chat::TestChat, test_user::TestUser, topic_card::TopicCard},
    services::Service,
};

#[function_component]
pub fn App() -> Html {
    let service = use_memo(|_| Service::default(), ());

    html! {
        <ContextProvider<Rc<Service>> context={service}>
            <Ui />
        </ContextProvider<Rc<Service>>>
    }
}

pub struct Ui {}

impl Component for Ui {
    type Message = ();

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = Service::from_context(ctx);

        spawn_local(async move {
            service
                .api
                .post_telemetry(
                    &serde_json::json!({
                        "event": "ui created"
                    }),
                    chameleon_protocol::http::TelemetryLevel::Info,
                )
                .await;
        });

        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
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
            <>
                <TopicCard {name} {secret_words} />
                <TestUser />
                <TestChat />
            </>
        }
    }
}
