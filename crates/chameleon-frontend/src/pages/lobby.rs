use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn Lobby(props: &Props) -> Html {
    html! {
        <div class="lobby">
            <div>{ "Lobby" }</div>
            <div>{ props.id.clone() }</div>
        </div>
    }
}
