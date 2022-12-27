use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{Document, Resource},
};
use yew::prelude::*;

use crate::services::Service;

use super::server_list_item::ServerListItem;

pub struct ServerList {
    games: Vec<Resource<GameAttributes>>,
}

pub enum Msg {
    FetchFailure(gloo::net::Error),
    FetchSuccess(Document<GameAttributes>),
}

impl Component for ServerList {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let this = Self { games: Vec::new() };
        this.fetch(ctx);
        this
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-list">
                <div class="server-list--header">
                    <ServerListItem
                        name=""
                        details=""
                        players="players"
                        password=false
                        selected=false />
                </div>
                <div class="server-list--body server-list--scrolling">
                {
                    self.games.iter().map(|game| {
                        let id = game.id.as_ref().unwrap().clone();
                        let name = game.attributes.as_ref().and_then(|a| a.name.as_ref()).unwrap().clone();
                        html! {
                            <ServerListItem
                                key={id}
                                name={name}
                                details="..."
                                players="..."
                                password=false
                                selected=false />
                        }
                    }).collect::<Html>()
                }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchFailure(error) => self.handle_fetch_failure(error),
            Msg::FetchSuccess(document) => self.handle_fetch_success(document),
        }
    }
}

impl ServerList {
    fn fetch(&self, ctx: &Context<Self>) {
        let service = Service::from_context(ctx);
        ctx.link().send_future(async move {
            match service.api.query_games(None).await {
                Ok(document) => Msg::FetchSuccess(document),
                Err(error) => Msg::FetchFailure(error),
            }
        });
    }

    fn handle_fetch_failure(&self, error: gloo::net::Error) -> bool {
        gloo::console::error!(format!("{error:?}"));
        true
    }

    fn handle_fetch_success(&mut self, document: Document<GameAttributes>) -> bool {
        if let Some(error) = document.errors {
            gloo::console::error!(format!("{error:?}"));
            return true;
        }

        let collection = document
            .try_get_resources()
            .expect("Expected resources")
            .try_get_collection()
            .expect("Expected collection")
            .clone();

        self.games.extend(collection);

        true
    }
}
