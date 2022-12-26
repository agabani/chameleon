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
                <div class="header">
                    <ServerListItem
                        name=""
                        details=""
                        players="players"
                        password=false
                        selected=false />
                </div>
                <div class="body scrolling">
                {
                    (0..=10).map(|_| {
                        html!{
                            <>
                                <ServerListItem
                                    name="Hardcore 2042"
                                    details="Manifest - Conquest Large"
                                    players="127 / 128"
                                    password=false
                                    selected=false />
                                <ServerListItem
                                    name="Human vs AI"
                                    details="Manifest - Conquest Large"
                                    players="127 / 128"
                                    password=false
                                    selected=true />
                                <ServerListItem
                                    name="TDM"
                                    details="Noshahr Canals - Custom"
                                    players="31 / 32"
                                    password=true
                                    selected=false />
                            </>
                        }
                    }).collect::<Html>()
                }
                </div>
            </div>
        }
    }
}
