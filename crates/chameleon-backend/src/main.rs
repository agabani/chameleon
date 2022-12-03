#![deny(clippy::pedantic)]

#[tokio::main]
async fn main() {
    chameleon_backend::app().await;
}
