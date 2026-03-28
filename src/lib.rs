#[macro_use]
extern crate lazy_static;

// Private modules
mod commands;
mod errors;
mod input;
mod models;
mod presenters;
mod util;
mod view;

// External application API
pub use anyhow::Error;
pub use crate::models::Application;
