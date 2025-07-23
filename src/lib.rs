// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod debug;

// Private modules
mod commands;
mod errors;
mod input;
mod models;
mod presenters;
mod util;
mod view;

// External application API
pub use crate::errors::Error;
pub use crate::models::Application;
