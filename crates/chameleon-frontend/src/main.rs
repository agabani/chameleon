#![deny(clippy::pedantic)]

fn main() {
    yew::Renderer::<chameleon_frontend::App>::new().render();
}
