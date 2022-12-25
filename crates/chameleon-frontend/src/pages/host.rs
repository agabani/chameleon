use yew::prelude::*;

use crate::components::top_menu::{Item, TopMenu};

pub struct Host {}

impl Component for Host {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <TopMenu active={Item::Host} />
        }
    }
}
