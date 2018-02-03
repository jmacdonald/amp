use std::fmt;

pub struct PathMode {
    pub input: String,
    pub save_on_accept: bool,
}

impl PathMode {
    pub fn new(initial_path: String) -> PathMode {
        PathMode {
            input: initial_path,
            save_on_accept: false
        }
    }
    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }
    pub fn pop_char(&mut self) {
        self.input.pop();
    }
}

impl fmt::Display for PathMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PATH")
    }
}
