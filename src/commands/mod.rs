use models::application::Application;
use errors;

pub mod application;
pub mod buffer;
pub mod cursor;
pub mod git;
pub mod jump_mode;
pub mod line_jump;
pub mod symbol_jump;
pub mod search;
pub mod selection;
pub mod search_select_mode;
pub mod theme;
pub mod view;
pub mod workspace;

pub type Command = fn(&mut Application) -> Result;
pub type Result = errors::Result<()>;
