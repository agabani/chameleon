#![deny(clippy::pedantic)]

use axum::{routing::get, Router};

#[allow(clippy::missing_panics_doc)]
pub async fn app() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
