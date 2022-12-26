use yew::prelude::*;

use super::server_list_item::ServerListItem;

pub struct ServerList {}

impl Component for ServerList {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-list">
                <ServerListItem />
                <ServerListItem />
                <ServerListItem />
            </div>
        }
    }
}
