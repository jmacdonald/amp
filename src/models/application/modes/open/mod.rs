mod displayable_path;
pub mod exclusions;

use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;
use bloodhound::ExclusionPattern;
use helpers::SelectableVec;
use models::application::modes::{SearchSelectMode, MAX_SEARCH_SELECT_RESULTS};
use std::sync::mpsc::{self, Receiver};
use std::thread;
pub use bloodhound::Index;
pub use self::displayable_path::DisplayablePath;

pub enum OpenModeIndex {
    Complete(Index),
    Indexing(Receiver<Index>)
}

pub struct OpenMode {
    pub insert: bool,
    pub input: String,
    index: OpenModeIndex,
    pub results: SelectableVec<DisplayablePath>,
}

impl OpenMode {
    pub fn new(path: PathBuf, exclusions: Option<Vec<ExclusionPattern>>) -> OpenMode {
        // Build and populate the index in a separate thread.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut index = Index::new(path);
            index.populate(exclusions, false);
            tx.send(index);
        });

        OpenMode {
            insert: true,
            input: String::new(),
            index: OpenModeIndex::Indexing(rx),
            results: SelectableVec::new(Vec::new()),
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
        let results =
            if let OpenModeIndex::Complete(ref index) = self.index {
                index.find(
                    &self.input.to_lowercase(),
                    MAX_SEARCH_SELECT_RESULTS
                ).into_iter()
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
}
