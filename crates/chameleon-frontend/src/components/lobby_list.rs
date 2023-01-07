use yew::prelude::*;

#[function_component]
pub fn LobbyList(props: &Props) -> Html {
    html! {
        <div class="lobby-list">
            { props.children.clone() }
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}
