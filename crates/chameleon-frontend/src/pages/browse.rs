use yew::prelude::*;

use crate::components::lobby_list_infinite_scrolling::LobbyListInfiniteScrolling;

#[function_component]
pub fn Browse() -> Html {
    let onclick = move |id| gloo::console::log!(format!("lobby {id} clicked"));

    html! {
        <div class="browse">
            <div>{ "Browse" }</div>
            <LobbyListInfiniteScrolling onclick={onclick} />
        </div>
    }
}
