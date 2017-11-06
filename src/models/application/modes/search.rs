pub struct SearchMode {
    pub input: String,
}

impl SearchMode {
    pub fn new() -> SearchMode {
        SearchMode { input: String::new() }
    }
}
