use yew::prelude::*;

use crate::components::lobby_list_infinite_scrolling::LobbyListInfiniteScrolling;

#[function_component]
pub fn Browse() -> Html {
    html! {
        <div class="browse">
            <div>{ "Browse" }</div>
            <LobbyListInfiniteScrolling />
        </div>
    }
}
