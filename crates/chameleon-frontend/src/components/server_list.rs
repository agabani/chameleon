use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{Document, Resource},
};
use yew::prelude::*;

use crate::services::Service;

use super::server_list_item::ServerListItem;

pub struct ServerList {
    games: Vec<Resource<GameAttributes>>,
    selected: Option<String>,
}

pub enum Msg {
    FetchFailure(gloo::net::Error),
    FetchSuccess(Document<GameAttributes>),
    ItemClicked(String),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onclick: Callback<String>,
}

impl Component for ServerList {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self::fetch(ctx);
        Self {
            games: Vec::new(),
            selected: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-list">
                <div class="server-list--header">
                    <ServerListItem
                        name=""
                        details=""
                        players="players"
                        password=false
                        selected=false
                        onclick={ Callback::from(|_| {}) } />
                </div>
                <div class="server-list--body server-list--scrolling">
                {
                    self.games.iter().map(|game| {
                        let id = game.id.as_ref().unwrap();
                        let name = game.attributes.as_ref().and_then(|a| a.name.as_ref()).unwrap().clone();

                        let id_ = id.clone();
                        let onclick = ctx.link().callback(move |_| Msg::ItemClicked(id_.clone()));

                        html! {
                            <ServerListItem
                                key={id.clone()}
                                name={name}
                                details="..."
                                players="..."
                                password=false
                                selected={ if let Some(selected) = &self.selected { selected == id } else { false }}
                                onclick={ onclick } />
                        }
                    }).collect::<Html>()
                }
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchFailure(error) => Self::handle_fetch_failure(&error),
            Msg::FetchSuccess(document) => self.handle_fetch_success(document),
            Msg::ItemClicked(id) => {
                ctx.props().onclick.emit(id.clone());
                self.selected = Some(id);
                true
            }
        }
    }
}

impl ServerList {
    fn fetch(ctx: &Context<Self>) {
        let service = Service::from_context(ctx);
        ctx.link().send_future(async move {
            match service.api.query_games(None).await {
                Ok(document) => Msg::FetchSuccess(document),
                Err(error) => Msg::FetchFailure(error),
            }
        });
    }

    fn handle_fetch_failure(error: &gloo::net::Error) -> bool {
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
