#![deny(clippy::pedantic)]

use tracing::Level;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, registry};

#[tokio::main]
async fn main() {
    let fmt = layer().pretty();
    let filter = LevelFilter::from_level(Level::INFO);
    registry().with(fmt).with(filter).init();

    chameleon_backend::app().await;
}
