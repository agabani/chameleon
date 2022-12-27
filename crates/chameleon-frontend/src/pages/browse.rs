use yew::prelude::*;

use crate::components::{
    server_details::ServerDetails,
    server_list::ServerList,
    top_menu::{Item, TopMenu},
};

pub struct Browse {
    selected: Option<String>,
}

pub enum Msg {
    ItemSelected(String),
}

impl Component for Browse {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { selected: None }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let item_selected = _ctx.link().callback(|id| Msg::ItemSelected(id));

        html! {
            <div class="browse">
                <TopMenu active={Item::Browse} />
                <div class="browse--content">
                    <div class="browse--smoke medium"></div>
                    <div class="browse--smoke small"></div>
                    <div class="browse--smoke large"></div>
                    <div class="browse--list">
                        <div class="browse--title">{ "Browse Servers" }</div>
                        <ServerList onclick={item_selected} />
                    </div>
                    <div class="browse--details">
                        {
                            if let Some(selected) = &self.selected {
                                html!{
                                    <ServerDetails
                                        id={ selected.clone() }
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
                                }
                            } else {
                                html!{}
                            }
                        }

                    </div>
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ItemSelected(id) => {
                self.selected = Some(id);
                true
            }
        }
    }
}
