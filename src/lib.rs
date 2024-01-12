// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

// Private modules
mod commands;
mod errors;
mod util;
mod input;
mod models;
mod presenters;
mod view;

// External application API
pub use crate::models::Application;
pub use crate::errors::Error;
