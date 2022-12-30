use yew::prelude::*;

use crate::components::{
    lobby_details::LobbyDetails, lobby_list_infinite_scrolling::LobbyListInfiniteScrolling,
};

#[function_component]
pub fn Browse() -> Html {
    let state = use_state(State::default);

    let onclick = {
        let state = state.clone();
        move |id: String| {
            state.set(State {
                selected: Some(id.into()),
            });
        }
    };

    html! {
        <div class="browse">
            <div>{ "Browse" }</div>
            <LobbyListInfiniteScrolling onclick={onclick} />
            {
                if let Some(selected) = &state.selected {
                    html! {
                        <LobbyDetails id={selected} />
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[derive(Default)]
struct State {
    selected: Option<AttrValue>,
}
