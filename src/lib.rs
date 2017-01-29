#![feature(associated_consts)]

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// External dependencies
extern crate git2;
extern crate luthor;
extern crate pad;
extern crate scribe;
extern crate regex;
extern crate syntect;

#[macro_use]
extern crate error_chain;

// Private modules
mod commands;
mod errors;
mod helpers;
mod input;
mod models;
mod presenters;
mod view;

// External application API
pub use models::Application;
