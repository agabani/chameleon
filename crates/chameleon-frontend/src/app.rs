use yew::prelude::*;

use crate::{
    components::{
        authentication_switch::AuthenticationSwitch, theme_container::ThemeContainer,
        theme_picker::ThemePicker,
    },
    contexts::{current_user::CurrentUserProvider, network::NetworkProvider, theme::ThemeProvider},
    pages::name::Name,
};

#[function_component]
pub fn App() -> Html {
    html! {
        <NetworkProvider>
        <ThemeProvider>
        <CurrentUserProvider>
            <ThemeContainer>
                <ThemePicker />
                <AuthenticationSwitch
                    challenge={|_| html!{ <Name /> }}
                    render={|_| html!{ { "app" } }} />
            </ThemeContainer>
        </CurrentUserProvider>
        </ThemeProvider>
        </NetworkProvider>
    }
}
