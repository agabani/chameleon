use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn LobbyMemberList(props: &Props) -> Html {
    html! {
        <div class="lobby-member-list">
            { props.children.clone() }
        </div>
    }
}
