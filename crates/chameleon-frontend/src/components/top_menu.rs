use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

pub struct TopMenu {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub active: Item,
}

#[derive(PartialEq)]
pub enum Item {
    Home,
    Browse,
    Host,
}

impl Component for TopMenu {
    type Message = ();

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let active = &ctx.props().active;

        let items = [
            (Item::Home, Route::Home, "Chameleon"),
            (Item::Browse, Route::Browse, "Browse"),
            (Item::Host, Route::Host, "Host"),
        ]
        .into_iter()
        .map(|(item, route, name)| {
            html! {
                <Link<Route>
                    classes={classes!(if item.eq(active) {"active".into()} else {None})}
                    key={name}
                    to={route}
                >{ name }</Link<Route>>
            }
        })
        .collect::<Vec<_>>();

        html! {
            <div class={classes!("top-menu")}>
                {items}
            </div>
        }
    }
}
