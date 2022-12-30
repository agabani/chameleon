use yew::prelude::*;

use crate::components::{lobby_list::LobbyList, lobby_list_item::LobbyListItem};

#[function_component]
pub fn Browse() -> Html {
    html! {
        <div class="browse">
            <div>{ "Browse" }</div>
            <LobbyList>
                <LobbyListItem id="1" name="My Lobby 1" />
                <LobbyListItem id="2" name="My Lobby 2" />
            </LobbyList>
        </div>
    }
}
