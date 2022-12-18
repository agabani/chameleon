#![deny(clippy::pedantic)]

use tracing_subscriber::{fmt::layer, prelude::*, registry, EnvFilter};

#[tokio::main]
async fn main() {
    registry()
        .with(layer().pretty())
        .with(EnvFilter::from_env("CHAMELEON_LOG"))
        .init();

    chameleon_backend::app().await;
}
