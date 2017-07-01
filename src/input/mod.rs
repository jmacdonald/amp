use std::clone::Clone;
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::hash::Hash;

pub use self::key_map::KeyMap;

pub mod modes;
mod key_map;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    Esc,
    Tab,
    Enter,
    AnyChar,
    Char(char),
    Ctrl(char),
}
