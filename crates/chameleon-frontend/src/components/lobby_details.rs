use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
    pub name: AttrValue,
    pub host: AttrValue,
    pub onclick: Callback<MouseEvent>,
}

#[function_component]
pub fn LobbyDetails(props: &Props) -> Html {
    html! {
        <div class="lobby-details">
            <div>{ &props.id }</div>
            <div>{ &props.name }</div>
            <div>{ &props.host }</div>
            <div>
                <button onclick={&props.onclick}>{ "join" }</button>
            </div>
        </div>
    }
}
