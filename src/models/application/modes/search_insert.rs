extern crate scribe;

pub struct SearchInsertMode {
    pub input: String,
}

pub fn new() -> SearchInsertMode {
    SearchInsertMode { input: String::new() }
}
