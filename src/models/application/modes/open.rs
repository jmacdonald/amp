extern crate bloodhound;

use std::path::PathBuf;
use self::bloodhound::Index;
use self::bloodhound::matching::Result;

const MAX_RESULTS: usize = 5;

pub struct OpenMode {
    pub input: String,
    index: Index,
    pub results: Vec<Result>,
    selected_result_index: usize,
}

impl OpenMode {
    pub fn selected_path(&self) -> Option<PathBuf> {
        match self.results.get(self.selected_result_index) {
            Some(ref result) => Some(result.path.clone()),
            None => None
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected_result_index
    }

    pub fn search(&mut self) {
        self.results = self.index.find(
            &self.input,         // The query string (needle).
            MAX_RESULTS          // Limit the amount of returned results.
        );
    }

    pub fn select_next_path(&mut self) {
        if self.selected_result_index < self.results.len() - 1 {
            self.selected_result_index += 1
        }
    }

    pub fn select_previous_path(&mut self) {
        if self.selected_result_index > 0 {
            self.selected_result_index -= 1
        }
    }
}

pub fn new(path: PathBuf) -> OpenMode {
    // Build and populate the index.
    let mut index = Index::new(path);
    index.populate();

    OpenMode{
        input: String::new(),
        index: index,
        results: Vec::new(),
        selected_result_index: 0
    }
}

#[cfg(test)]
mod tests {
    use super::new;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn selected_path_returns_none_when_results_are_empty() {
        let mode = super::new(env::current_dir().unwrap());
        assert_eq!(mode.selected_path(), None);
    }

    #[test]
    fn selected_path_returns_correct_entry_when_there_are_results() {
        let mut mode = super::new(env::current_dir().unwrap());
        mode.input = "Cargo.toml".to_string();
        mode.search();
        assert_eq!(mode.selected_path(), Some(PathBuf::from(mode.input)));
    }

    #[test]
    fn select_next_path_advances_until_the_end_of_the_result_set() {
        let mut mode = super::new(env::current_dir().unwrap());
        mode.input = "Cargo.toml".to_string();
        mode.search();
        assert_eq!(mode.selected_index(), 0);

        for _ in 0..10 {
            mode.select_next_path()
        }
        assert_eq!(mode.selected_index(), 4);
    }

    #[test]
    fn select_previous_path_reverses_until_the_start_of_the_result_set() {
        let mut mode = super::new(env::current_dir().unwrap());
        mode.input = "Cargo.toml".to_string();
        mode.search();

        // Advance the selection.
        for _ in 0..4 {
            mode.select_next_path()
        }
        assert_eq!(mode.selected_index(), 4);

        // Reverse the selection.
        for _ in 0..10 {
            mode.select_previous_path()
        }
        assert_eq!(mode.selected_index(), 0);
    }
}
