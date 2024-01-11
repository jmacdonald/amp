// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// External dependencies
extern crate app_dirs2;
extern crate bloodhound;
extern crate fragment;
extern crate git2;
extern crate luthor;
extern crate mio;
extern crate regex;
extern crate scribe;
extern crate signal_hook;
extern crate syntect;
extern crate unicode_segmentation;
extern crate cli_clipboard as clipboard;
extern crate yaml_rust as yaml;
extern crate smallvec;

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
