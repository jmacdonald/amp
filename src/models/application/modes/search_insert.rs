extern crate scribe;

pub struct SearchInsertMode {
    pub input: String,
}

impl SearchInsertMode {
    pub fn new() -> SearchInsertMode {
        SearchInsertMode { input: String::new() }
    }
}
