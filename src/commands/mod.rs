use models::application::Application;

pub mod application;
pub mod workspace;
pub mod buffer;
pub mod cursor;
pub mod selection;
pub mod jump_mode;
pub mod line_jump;
pub mod open_mode;
pub mod search;
pub mod view;

pub type Command = fn(&mut Application);
