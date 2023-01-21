#![deny(clippy::pedantic)]

mod app;
mod args;
mod database;
mod domain_old;
mod error;
mod extract;
mod routes;

pub use app::app;
