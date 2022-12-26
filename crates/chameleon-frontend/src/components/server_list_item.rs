use yew::prelude::*;

pub struct ServerListItem {}

impl Component for ServerListItem {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-list-item"></div>
        }
    }
}
