mod displayable_path;
pub mod exclusions;

pub use self::displayable_path::DisplayablePath;
use crate::models::application::modes::{PopSearchToken, SearchSelectConfig, SearchSelectMode};
use crate::models::application::Event;
use crate::util::SelectableVec;
use bloodhound::ExclusionPattern;
pub use bloodhound::Index;
use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;
use std::sync::mpsc::Sender;
use std::thread;

#[derive(PartialEq)]
pub enum OpenModeIndex {
    Complete(Index),
    Indexing(PathBuf),
}

pub struct OpenMode {
    pub insert: bool,
    input: String,
    pinned_input: String,
    index: OpenModeIndex,
    pub results: SelectableVec<DisplayablePath>,
    config: SearchSelectConfig,
}

impl OpenMode {
    pub fn new(path: PathBuf, config: SearchSelectConfig) -> OpenMode {
        OpenMode {
            insert: true,
            input: String::new(),
            pinned_input: String::new(),
            index: OpenModeIndex::Indexing(path),
            results: SelectableVec::new(Vec::new()),
            config,
        }
    }

    pub fn set_index(&mut self, index: Index) {
        self.index = OpenModeIndex::Complete(index)
    }

    pub fn reset(
        &mut self,
        path: PathBuf,
        exclusions: Option<Vec<ExclusionPattern>>,
        events: Sender<Event>,
        config: SearchSelectConfig,
    ) {
        self.insert = true;
        self.input.clear();
        self.config = config;
        self.index = OpenModeIndex::Indexing(path.clone());
        self.results = SelectableVec::new(Vec::new());

        // Build and populate the index in a separate thread.
        thread::spawn(move || {
            let mut index = Index::new(path);
            index.populate(exclusions, false);
            let _ = events.send(Event::OpenModeIndexComplete(index));
        });
    }

    pub fn pinned_query(&self) -> &str {
        &self.pinned_input
    }

    pub fn pin_query(&mut self) {
        // Normalize whitespace between tokens
        for token in self.input.split_whitespace() {
            if !self.pinned_input.is_empty() {
                self.pinned_input.push(' ');
            }

            self.pinned_input.push_str(token);
        }

        self.input.truncate(0);
    }

    pub fn pop_search_token(&mut self) {
        if self.input.is_empty() {
            if self.pinned_input.is_empty() {
                return;
            }

            // Find the last word boundary (transition to/from whitespace), using
            // using fold to carry the previous character's type forward.
            let mut boundary_index = 0;
            self.pinned_input
                .char_indices()
                .fold(true, |was_whitespace, (index, c)| {
                    if index > 0 && c.is_whitespace() != was_whitespace {
                        boundary_index = index - 1;
                    }

                    c.is_whitespace()
                });

            self.pinned_input.truncate(boundary_index);
        } else {
            // Call the default implementation
            PopSearchToken::pop_search_token(self);
        }
    }
}

impl fmt::Display for OpenMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OPEN")
    }
}

impl SearchSelectMode for OpenMode {
    type Item = DisplayablePath;

    fn search(&mut self) {
        let results = if let OpenModeIndex::Complete(ref index) = self.index {
            index
                .find(
                    &format!(
                        "{} {}",
                        self.pinned_input.to_lowercase(),
                        self.input.to_lowercase()
                    ),
                    self.config.max_results,
                )
                .into_iter()
                .map(|path| DisplayablePath(path.to_path_buf()))
                .collect()
        } else {
            vec![]
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

    fn results(&self) -> Iter<DisplayablePath> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&DisplayablePath> {
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

    fn message(&mut self) -> Option<String> {
        if let OpenModeIndex::Indexing(ref path) = self.index {
            Some(format!("Indexing {}", path.to_string_lossy()))
        } else if self.pinned_query().is_empty() && self.query().is_empty() {
            Some(String::from("Enter a search query to start."))
        } else if self.results().count() == 0 {
            Some(String::from("No matching entries found."))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpenMode;
    use crate::models::application::modes::{SearchSelectConfig, SearchSelectMode};
    use crate::models::application::Event;
    use std::env;
    use std::sync::mpsc::channel;

    #[test]
    fn search_uses_the_query() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(path, None, sender, config);
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo.toml");
        mode.search();

        let results: Vec<String> = mode.results().map(|r| r.to_string()).collect();
        assert_eq!(results, vec!["Cargo.toml"]);
    }

    #[test]
    fn pin_query_transfers_content() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());

        mode.query().push_str("Cargo");
        mode.pin_query();

        assert_eq!(mode.query(), "");
        assert_eq!(mode.pinned_query(), "Cargo");
    }

    #[test]
    fn pin_query_trims_content() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());

        mode.query().push_str(" Cargo ");
        mode.pin_query();

        assert_eq!(mode.query(), "");
        assert_eq!(mode.pinned_query(), "Cargo");
    }

    #[test]
    fn pin_query_normalizes_whitespace() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());

        mode.query().push_str("amp  editor");
        mode.pin_query();

        assert_eq!(mode.query(), "");
        assert_eq!(mode.pinned_query(), "amp editor");
    }

    #[test]
    fn subsequent_pin_query_accumulates_content() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());

        mode.query().push_str("Cargo");
        mode.pin_query();
        mode.query().push_str("toml");
        mode.pin_query();

        assert_eq!(mode.query(), "");
        assert_eq!(mode.pinned_query(), "Cargo toml"); // space is intentional
    }

    #[test]
    fn search_incorporates_pinned_query_content() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(path, None, sender, config);
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("toml");
        mode.pin_query();
        mode.query().push_str("Cargo");
        mode.search();

        let results: Vec<String> = mode.results().map(|r| r.to_string()).collect();
        assert_eq!(results, vec!["Cargo.toml"]);
    }

    #[test]
    fn pop_search_token_eats_into_pinned_query_when_query_is_empty() {
        let path = env::current_dir().expect("can't get current directory/path");
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());

        mode.query().push_str("two tokens");
        mode.pin_query();
        mode.pop_search_token();

        assert_eq!(mode.pinned_query(), "two");
        mode.pop_search_token();
        assert_eq!(mode.pinned_query(), "");
    }
}
