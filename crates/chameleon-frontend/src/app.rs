use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{
        authentication_switch::AuthenticationSwitch, theme_container::ThemeContainer,
        theme_picker::ThemePicker,
    },
    contexts::{current_user::CurrentUserProvider, network::NetworkProvider, theme::ThemeProvider},
    pages::{browse::Browse, host::Host, main_menu::MainMenu, name::Name},
};

#[function_component]
pub fn App() -> Html {
    html! {
        <NetworkProvider>
        <ThemeProvider>
        <CurrentUserProvider>
        <BrowserRouter>
            <ThemeContainer>
                <ThemePicker />
                <Switch<Route> render={|route| match route {
                    Route::MainMenu => html! {
                        <AuthenticationSwitch
                            challenge={|_| html!{ <Name /> }}
                            render={|_| html!{ <MainMenu /> }}
                        />
                    },
                    Route::Browse =>  html !{
                        <AuthenticationSwitch
                            challenge={|_| html!{ <Name /> }}
                            render={|_| html!{ <Browse /> }}
                        />
                    },
                    Route::Host =>  html !{
                        <AuthenticationSwitch
                            challenge={|_| html!{ <Name /> }}
                            render={|_| html!{ <Host /> }}
                        />
                    },
                    Route::NotFound => html !{
                        <div>{ "Not Found" }</div>
                    },
                }} />
            </ThemeContainer>
        </BrowserRouter>
        </CurrentUserProvider>
        </ThemeProvider>
        </NetworkProvider>
    }
}

#[derive(Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    MainMenu,
    #[at("/browse")]
    Browse,
    #[at("/host")]
    Host,
    #[not_found]
    #[at("/not-found")]
    NotFound,
}
