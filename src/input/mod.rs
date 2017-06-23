use std::clone::Clone;
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::hash::Hash;

pub mod modes;

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
    Char(char),
    Ctrl(char),
}
