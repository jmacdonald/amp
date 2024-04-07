use std::fmt;

#[derive(Default)]
pub struct PathMode {
    pub input: String,
    pub save_on_accept: bool,
}

impl PathMode {
    pub fn new() -> PathMode {
        PathMode::default()
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn reset(&mut self, initial_path: String) {
        self.input = initial_path;
        self.save_on_accept = false;
    }
}

impl fmt::Display for PathMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PATH")
    }
}
