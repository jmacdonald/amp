use fragment;
use crate::util::SelectableVec;
use std::fmt;
use std::slice::Iter;
use crate::models::application::modes::{SearchSelectMode, SearchSelectConfig};

pub struct ThemeMode {
    insert: bool,
    input: String,
    themes: Vec<String>,
    results: SelectableVec<String>,
    config: SearchSelectConfig,
}

impl ThemeMode {
    pub fn new(themes: Vec<String>, config: SearchSelectConfig) -> ThemeMode {
        ThemeMode {
            insert: true,
            input: String::new(),
            themes,
            results: SelectableVec::new(Vec::new()),
            config,
        }
    }
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "THEME")
    }
}

impl SearchSelectMode<String> for ThemeMode {
    fn search(&mut self) {
        // Find the themes we're looking for using the query.
        let results = fragment::matching::find(&self.input, &self.themes, self.config.max_results);

        // We don't care about the result objects; we just want
        // the underlying symbols. Map the collection to get these.
        self.results = SelectableVec::new(
            results
            .into_iter()
            .map(|r| r.clone())
            .collect()
        );
    }

    fn query(&mut self) -> &mut String {
        &mut self.input
    }

    fn insert_mode(&self) -> bool {
        self.insert
    }

    fn set_insert_mode(&mut self, insert_mode: bool) {
        self.insert = insert_mode;
    }

    fn results(&self) -> Iter<String> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&String> {
        self.results.selection()
    }

    fn selected_index(&self) -> usize {
        self.results.selected_index()
    }

    fn select_previous(&mut self) {
        self.results.select_previous();
    }

    fn select_next(&mut self) {
        self.results.select_next();
    }

    fn config(&self) -> &SearchSelectConfig {
        &self.config
    }
}
