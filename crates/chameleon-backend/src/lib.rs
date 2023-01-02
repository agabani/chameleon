#![deny(clippy::pedantic)]

mod app;
mod args;
mod database;
mod domain;
mod error;
mod extract;
mod routes;

pub use app::app;
