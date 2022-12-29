use std::rc::Rc;

use yew::prelude::*;

#[derive(Default, PartialEq)]
pub struct CurrentUserState {
    authenticated: bool,
}

impl Reducible for CurrentUserState {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}

pub type CurrentUserContext = UseReducerHandle<CurrentUserState>;

#[derive(PartialEq, Properties)]
pub struct CurrentUserProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn CurrentUserProvider(props: &CurrentUserProps) -> Html {
    let context = use_reducer(CurrentUserState::default);

    html! {
        <ContextProvider<CurrentUserContext> context={context}>
            { props.children.clone() }
        </ContextProvider<CurrentUserContext>>
    }
}
