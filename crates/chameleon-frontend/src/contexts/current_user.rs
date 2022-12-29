use std::rc::Rc;

use yew::prelude::*;

#[derive(Default, PartialEq)]
pub struct CurrentUserState {
    pub authenticated: bool,
}

pub type CurrentUserContext = UseReducerHandle<CurrentUserState>;

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn CurrentUserProvider(props: &Props) -> Html {
    let context = use_reducer(CurrentUserState::default);

    html! {
        <ContextProvider<CurrentUserContext> context={context}>
            { props.children.clone() }
        </ContextProvider<CurrentUserContext>>
    }
}

impl Reducible for CurrentUserState {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}
