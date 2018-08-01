// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// External dependencies
extern crate app_dirs;
extern crate bloodhound;
extern crate fragment;
extern crate git2;
extern crate luthor;
extern crate pad;
extern crate regex;
extern crate scribe;
extern crate syntect;
extern crate unicode_segmentation;
extern crate clipboard;
extern crate yaml_rust as yaml;
extern crate smallvec;

#[macro_use]
extern crate error_chain;

// Private modules
mod commands;
mod errors;
mod util;
mod input;
mod models;
mod presenters;
mod view;

// External application API
pub use models::Application;
pub use errors::Error;
