use yew::prelude::*;

pub struct ServerDetails {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
    pub name: AttrValue,
    pub host: AttrValue,
    pub modifiers: Vec<AttrValue>,
}

impl Component for ServerDetails {
    type Message = ();

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-details">
                <div class="server-details--title">{ &ctx.props().name }</div>
                <div class="server-details--hosting">
                    <div>{ "Hosted by:" }</div>
                    <div class="server-details--host">{ &ctx.props().host }</div>
                </div>
                <div class="server-details--actions">
                    <button class="server-details--action">{ "Join Match" }</button>
                    <button class="server-details--action">{ "Request Invite" }</button>
                </div>
                <div class="server-details--modifiers">
                    {
                        ctx.props().modifiers.iter().map(|modifier| {
                            html! {
                                <div class="server-details--modifier">{ modifier }</div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>
        }
    }
}
