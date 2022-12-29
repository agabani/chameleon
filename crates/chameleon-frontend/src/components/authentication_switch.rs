use yew::prelude::*;

use crate::contexts::current_user::CurrentUserContext;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub challenge: Callback<(), Html>,
    pub render: Callback<(), Html>,
}

#[function_component]
pub fn AuthenticationSwitch(props: &Props) -> Html {
    let context = use_context::<CurrentUserContext>().unwrap();

    if context.authenticated {
        props.render.emit(())
    } else {
        props.challenge.emit(())
    }
}
