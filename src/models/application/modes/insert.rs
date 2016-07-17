pub struct InsertMode {
    pub input: Option<char>,
}

impl InsertMode {
    pub fn new() -> InsertMode {
        InsertMode { input: None }
    }
}
