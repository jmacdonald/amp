use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;
use helpers::SelectableSet;
use models::application::modes::SearchSelectMode;
use bloodhound::Index;

pub struct OpenMode {
    pub insert: bool,
    pub input: String,
    index: Index,
    pub results: SelectableSet<DisplayablePath>,
}

pub struct DisplayablePath(pub PathBuf);

impl fmt::Display for DisplayablePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &DisplayablePath(ref path) = self;
        write!(f, "{}", path.to_string_lossy())
    }
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
}

impl SearchSelectMode<DisplayablePath> for OpenMode {
    fn search(&mut self) {
        let results = self.index.find(
            &self.input,
            OpenMode::MAX_RESULTS
        ).into_iter().map(|path| DisplayablePath(path)).collect();
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
