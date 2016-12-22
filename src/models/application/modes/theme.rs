extern crate fragment;

use helpers::SelectableSet;
use std::iter::Iterator;

pub struct ThemeMode {
    pub insert: bool,
    pub input: String,
    pub themes: Vec<String>,
    pub results: SelectableSet<String>,
}

impl ThemeMode {
    pub const MAX_RESULTS: usize = 5;

    pub fn new(themes: Vec<String>) -> ThemeMode {
        ThemeMode {
            insert: true,
            input: String::new(),
            themes: themes,
            results: SelectableSet::new(Vec::new()),
        }
    }

    pub fn search(&mut self) {
        // Find the themes we're looking for using the query.
        let results = fragment::matching::find(&self.input, &self.themes, ThemeMode::MAX_RESULTS);

        // We don't care about the result objects; we just want
        // the underlying symbols. Map the collection to get these.
        self.results = SelectableSet::new(
            results
            .into_iter()
            .map(|r| r.clone())
            .collect()
        );
    }
}
