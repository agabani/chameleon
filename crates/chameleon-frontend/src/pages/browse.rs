use yew::prelude::*;

use crate::components::{
    lobby_details::LobbyDetails, lobby_list_infinite_scrolling::LobbyListInfiniteScrolling,
};

#[function_component]
pub fn Browse() -> Html {
    let state = use_reducer(State::default);

    let onclick = {
        let state = state.clone();
        move |id: String| {
            state.dispatch(State {
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
                        <LobbyDetails id={selected} name="a" host="b" />
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

impl Reducible for State {
    type Action = Self;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        action.into()
    }
}
