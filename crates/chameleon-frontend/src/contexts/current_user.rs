use std::rc::Rc;

use yew::prelude::*;

#[derive(Default, PartialEq)]
pub struct State {
    authenticated: bool,
}

impl Reducible for State {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}

pub type Context = UseReducerHandle<State>;

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn Provider(props: &Props) -> Html {
    let context = use_reducer(State::default);

    html! {
        <ContextProvider<Context> context={context}>
            { props.children.clone() }
        </ContextProvider<Context>>
    }
}
