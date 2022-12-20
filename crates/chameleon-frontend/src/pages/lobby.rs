use yew::prelude::*;

pub struct Lobby {}

impl Component for Lobby {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <h1>{ "Lobby" }</h1>
        }
    }
}
