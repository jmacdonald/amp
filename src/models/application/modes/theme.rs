use fragment;
use helpers::SelectableSet;
use std::slice::Iter;
use models::application::modes::SearchSelectMode;

pub struct ThemeMode {
    insert: bool,
    input: String,
    themes: Vec<String>,
    results: SelectableSet<String>,
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
}

impl SearchSelectMode<String> for ThemeMode {
    fn search(&mut self) {
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
}
