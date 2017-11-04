mod displayable_path;
pub mod exclusions;

use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;
use bloodhound::ExclusionPattern;
use helpers::SelectableSet;
use models::application::modes::{SearchSelectMode, MAX_SEARCH_SELECT_RESULTS};
use bloodhound::Index;
pub use self::displayable_path::DisplayablePath;

pub struct OpenMode {
    pub insert: bool,
    pub input: String,
    index: Index,
    pub results: SelectableSet<DisplayablePath>,
}

impl OpenMode {
    pub fn new(path: PathBuf, exclusions: Option<Vec<ExclusionPattern>>) -> OpenMode {
        // Build and populate the index.
        let mut index = Index::new(path);
        index.populate(exclusions, false);

        OpenMode {
            insert: true,
            input: String::new(),
            index: index,
            results: SelectableSet::new(Vec::new()),
        }
    }
}

impl fmt::Display for OpenMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OPEN")
    }
}

impl SearchSelectMode<DisplayablePath> for OpenMode {
    fn search(&mut self) {
        let results = self.index.find(
            &self.input,
            MAX_SEARCH_SELECT_RESULTS
        ).into_iter().map(|path| DisplayablePath(path.to_path_buf())).collect();
        self.results = SelectableSet::new(results);
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
}
