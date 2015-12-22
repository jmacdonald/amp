extern crate scribe;

pub struct LineJumpMode {
    pub input: String,
}

pub fn new() -> LineJumpMode {
    LineJumpMode { input: String::new() }
}
