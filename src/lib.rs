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

#[macro_use]
mod helpers;

// Private modules
mod models;
mod view;
mod errors;
mod input;
mod commands;
mod presenters;

// External application API
pub use models::Application;
