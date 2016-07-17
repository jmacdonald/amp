extern crate bloodhound;

use std::path::PathBuf;
use helpers::SelectableSet;
use self::bloodhound::Index;

pub struct OpenMode {
    pub insert: bool,
    pub input: String,
    index: Index,
    pub results: SelectableSet<PathBuf>,
}

impl OpenMode {
    pub const MAX_RESULTS: usize = 5;

    pub fn new(path: PathBuf) -> OpenMode {
        // Build and populate the index.
        let mut index = Index::new(path);
        index.populate();

        OpenMode {
            insert: true,
            input: String::new(),
            index: index,
            results: SelectableSet::new(Vec::new()),
        }
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.results.selection()
    }

    pub fn search(&mut self) {
        let results = self.index.find(&self.input, // The query string (needle).
                                      OpenMode::MAX_RESULTS /* Limit the amount of returned results. */);
        self.results = SelectableSet::new(results);
    }
}
