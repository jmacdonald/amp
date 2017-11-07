pub struct SearchMode {
    pub insert: bool,
    pub input: String,
}

impl SearchMode {
    pub fn new() -> SearchMode {
        SearchMode {
            insert: true,
            input: String::new()
        }
    }

    pub fn insert_mode(&self) -> bool {
        self.insert
    }
}
