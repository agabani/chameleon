use yew::prelude::*;

use crate::components::{
    server_details::ServerDetails,
    server_list::ServerList,
    top_menu::{Item, TopMenu},
};

pub struct Browse {}

impl Component for Browse {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="browse">
                <TopMenu active={Item::Browse} />
                <div class="browse--content">
                    <div class="browse--smoke medium"></div>
                    <div class="browse--smoke small"></div>
                    <div class="browse--smoke large"></div>
                    <div class="browse--list">
                        <div class="browse--title">{ "Browse Servers" }</div>
                        <ServerList />
                    </div>
                    <div class="browse--details">
                        <ServerDetails
                            name="Hardcore 2042"
                            host="Ahn Bo Hyun"
                            modifiers={vec![
                                "Disappearing Hints".into(),
                                "Unlimited Discussion".into(),
                                "5 Rounds".into(),
                                "Custom Topic Cards".into(),
                                "Dice".into(),
                                "Shuffled Secrets".into(),
                                "Spectators".into(),
                            ]} />
                    </div>
                </div>
            </div>
        }
    }
}
