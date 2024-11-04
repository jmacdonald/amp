use crate::models::application::modes::{SearchSelectConfig, SearchSelectMode};
use crate::util::SelectableVec;
use fragment;
use std::fmt;
use std::slice::Iter;

#[derive(Default)]
pub struct ThemeMode {
    insert: bool,
    input: String,
    themes: Vec<String>,
    results: SelectableVec<String>,
    config: SearchSelectConfig,
}

impl ThemeMode {
    pub fn new(config: SearchSelectConfig) -> ThemeMode {
        ThemeMode {
            config,
            insert: true,
            ..Default::default()
        }
    }

    pub fn reset(&mut self, themes: Vec<String>, config: SearchSelectConfig) {
        *self = ThemeMode {
            config,
            insert: true,
            themes,
            ..Default::default()
        };
    }
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "THEME")
    }
}

impl SearchSelectMode for ThemeMode {
    type Item = String;

    fn search(&mut self) {
        // Find the themes we're looking for using the query.
        let results = if self.input.is_empty() {
            self.themes
                .iter()
                .take(self.config.max_results)
                .cloned()
                .collect()
        } else {
            fragment::matching::find(&self.input, &self.themes, self.config.max_results)
                .into_iter()
                .map(|i| i.clone())
                .collect()
        };

        self.results = SelectableVec::new(results);
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
