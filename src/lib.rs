#![feature(associated_consts)]

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// External dependencies
extern crate bloodhound;
extern crate fragment;
extern crate git2;
extern crate luthor;
extern crate pad;
extern crate regex;
extern crate scribe;
extern crate syntect;
extern crate rustc_serialize;
extern crate preferences;

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
