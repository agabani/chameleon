use chameleon_protocol::{attributes::GameAttributes, jsonapi::Document};
use yew::prelude::*;

use crate::services::Service;

pub struct ServerDetails {
    host: Option<AttrValue>,
    modifiers: Option<Vec<AttrValue>>,
    name: Option<AttrValue>,
}

pub enum Msg {
    FetchFailure(gloo::net::Error),
    FetchSuccess(Document<GameAttributes>),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

impl Component for ServerDetails {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self::fetch(ctx);
        Self {
            host: None,
            modifiers: None,
            name: None,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-details">
                <div class="server-details--title">{ self.name.as_ref() }</div>
                <div class="server-details--hosting">
                    <div>{ "Hosted by:" }</div>
                    <div class="server-details--host">{ self.host.as_ref() }</div>
                </div>
                <div class="server-details--actions">
                    <button class="server-details--action">{ "Join Match" }</button>
                    <button class="server-details--action">{ "Request Invite" }</button>
                </div>
                <div class="server-details--modifiers">
                    {
                        if let Some(modifiers) = &self.modifiers {
                            modifiers.iter().map(|modifier| html! {
                                <div class="server-details--modifier">{ modifier }</div>
                            }).collect::<Html>()
                        } else {
                            html!()
                        }
                    }
                </div>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchFailure(error) => Self::handle_fetch_failure(&error),
            Msg::FetchSuccess(document) => self.handle_fetch_success(document),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if ctx.props().id != old_props.id {
            Self::fetch(ctx);
        }
        true
    }
}

impl ServerDetails {
    fn fetch(ctx: &Context<Self>) {
        let id = ctx.props().id.clone();
        let service = Service::from_context(ctx);
        ctx.link().send_future(async move {
            match service.api.get_game(&id).await {
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

        let individual = document
            .try_get_resources()
            .expect("Expected resources")
            .try_get_individual()
            .expect("Expected individual")
            .clone();

        let name = individual
            .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
            .expect("Expected name")
            .clone();

        self.name = Some(name.into());

        let host = individual
            .try_get_relationship("host", "Host")
            .expect("Expected relationship")
            .try_get_resource_identifiers("host")
            .expect("Expected resource identifiers")
            .try_get_individual("host", "Host")
            .expect("Expected individual")
            .try_get_field(|a| a.id.as_ref(), "id", "Id")
            .expect("Expected Id")
            .clone();

        self.host = Some(host.into());

        true
    }
}
