use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{Resource, ResourcesDocument},
};
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::services::Service;

use super::server_list_item::ServerListItem;

pub struct ServerList {
    body: NodeRef,
    games: Vec<Resource<GameAttributes>>,
    next: Option<String>,
    next_visible: bool,
    selected: Option<String>,
}

pub enum Msg {
    FetchFailure(gloo::net::Error),
    FetchSuccess(ResourcesDocument<GameAttributes>),
    ItemClicked(String),
    LoadMoreClicked,
    Scrolled(i32, i32),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onclick: Callback<String>,
}

impl Component for ServerList {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let this = Self {
            body: NodeRef::default(),
            games: Vec::new(),
            next: None,
            next_visible: false,
            selected: None,
        };
        this.fetch(ctx);
        this
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let body = self.body.clone();
        let onclick = ctx.link().callback(|_| Msg::LoadMoreClicked);
        let onscroll = ctx.link().callback(move |_| {
            let element = body.cast::<web_sys::HtmlElement>().unwrap();
            Msg::Scrolled(
                element.scroll_top() + element.client_height(),
                element.scroll_height(),
            )
        });

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
                <div class="server-list--body server-list--scrolling" {onscroll} ref={self.body.clone()}>
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
                {
                    if self.next_visible {
                        html! {
                            <div class="server-list--load-more" {onclick}>{"Load More"}</div>
                        }
                    } else {
                        html!()
                    }
                }
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchFailure(error) => Self::handle_fetch_failure(&error),
            Msg::FetchSuccess(document) => self.handle_fetch_success(document),
            Msg::LoadMoreClicked => {
                self.fetch(ctx);
                false
            }
            Msg::ItemClicked(id) => {
                ctx.props().onclick.emit(id.clone());
                self.selected = Some(id);
                true
            }
            Msg::Scrolled(current, maximum) => {
                if maximum - current == 0 {
                    self.fetch(ctx);
                }
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let element = self.body.cast::<HtmlElement>().unwrap();

        if element.scroll_top() + element.client_height() == element.scroll_height() {
            self.next_visible = true;
        }
    }
}

impl ServerList {
    fn fetch(&self, ctx: &Context<Self>) {
        let next = self.next.clone();
        let service = Service::from_context(ctx);
        ctx.link().send_future(async move {
            match service.api.query_games(next).await {
                Ok(document) => Msg::FetchSuccess(document),
                Err(error) => Msg::FetchFailure(error),
            }
        });
    }

    fn handle_fetch_failure(error: &gloo::net::Error) -> bool {
        gloo::console::error!(format!("{error:?}"));
        true
    }

    fn handle_fetch_success(&mut self, document: ResourcesDocument<GameAttributes>) -> bool {
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

        if self.next.is_some() {
            self.next_visible = collection.is_empty();
        }

        self.games.extend(collection);

        let next = document
            .try_get_link("next", "Next")
            .expect("Expected links");

        self.next = Some(next.clone());

        true
    }
}
