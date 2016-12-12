use std::error;
use std::result;
use models::application::Application;

pub mod application;
pub mod buffer;
pub mod cursor;
pub mod git;
pub mod jump_mode;
pub mod line_jump;
pub mod symbol_jump;
pub mod open_mode;
pub mod search;
pub mod selection;
pub mod view;
pub mod workspace;

pub type Command = fn(&mut Application) -> Result;
pub type Result = result::Result<(), Box<error::Error>>;
