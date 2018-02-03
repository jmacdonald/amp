use errors::*;
use helpers::SelectableVec;
use std::fmt;
use scribe::buffer::{Buffer, Distance, Range};
use std::path::PathBuf;

pub struct NameBuffer {
    pub input: String,
}

impl NameBuffer {
    pub fn new(file_name: String) -> NameBuffer {
        NameBuffer {
            input: file_name
        }
    }
    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }
    pub fn pop_char(&mut self) {
        self.input.pop();
    }
    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.input)
    }
}

impl fmt::Display for NameBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NAME BUFFER")
    }
}
