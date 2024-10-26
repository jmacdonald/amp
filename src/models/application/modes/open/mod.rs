mod displayable_path;
pub mod exclusions;

pub use self::displayable_path::DisplayablePath;
use crate::errors::*;
use crate::models::application::modes::{PopSearchToken, SearchSelectConfig, SearchSelectMode};
use crate::models::application::Event;
use crate::util::SelectableVec;
use bloodhound::ExclusionPattern;
pub use bloodhound::Index;
use scribe::Workspace;
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
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
    buffers: SelectableVec<DisplayablePath>,
    pub results: SelectableVec<DisplayablePath>,
    marked_results: HashSet<usize>,
    config: SearchSelectConfig,
}

impl OpenMode {
    pub fn new(path: PathBuf, config: SearchSelectConfig) -> OpenMode {
        OpenMode {
            insert: true,
            input: String::new(),
            pinned_input: String::new(),
            index: OpenModeIndex::Indexing(path),
            buffers: SelectableVec::new(Vec::new()),
            results: SelectableVec::new(Vec::new()),
            marked_results: HashSet::new(),
            config,
        }
    }

    pub fn set_index(&mut self, index: Index) {
        self.index = OpenModeIndex::Complete(index)
    }

    pub fn reset(
        &mut self,
        workspace: &mut Workspace,
        exclusions: Option<Vec<ExclusionPattern>>,
        events: Sender<Event>,
        config: SearchSelectConfig,
    ) -> Result<()> {
        self.insert = true;
        self.input.clear();
        self.config = config;
        self.index = OpenModeIndex::Indexing(workspace.path.clone());
        self.buffers = SelectableVec::new(
            workspace
                .buffer_paths()
                .into_iter()
                .map(|p| {
                    let path = p.unwrap_or(Path::new("untitled"));

                    DisplayablePath(path.into())
                })
                .collect(),
        );
        if let Some(i) = workspace.current_buffer_index() {
            self.buffers.set_selected_index(i)?;
        }
        self.results = SelectableVec::new(Vec::new());
        self.marked_results = HashSet::new();

        // Build and populate the index in a separate thread.
        let path = workspace.path.clone();
        thread::spawn(move || {
            let mut index = Index::new(path);
            index.populate(exclusions, false);
            let _ = events.send(Event::OpenModeIndexComplete(index));
        });

        Ok(())
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

    pub fn toggle_selection(&mut self) {
        if self.marked_results.take(&self.selected_index()).is_none() {
            self.marked_results.insert(self.selected_index());
        }
    }

    pub fn selections(&self) -> Vec<&DisplayablePath> {
        let mut selections: Vec<&DisplayablePath> = self
            .marked_results
            .iter()
            .map(|i| self.results.get(*i).unwrap())
            .collect();
        selections.push(self.selection().unwrap());

        selections
    }

    pub fn selected_indices(&self) -> Vec<usize> {
        let mut selected_indices: Vec<usize> = self.marked_results.iter().copied().collect();
        selected_indices.push(self.selected_index());

        selected_indices
    }

    fn collection(&self) -> &SelectableVec<DisplayablePath> {
        if self.input.is_empty() {
            &self.buffers
        } else {
            &self.results
        }
    }

    fn collection_mut(&mut self) -> &mut SelectableVec<DisplayablePath> {
        if self.input.is_empty() {
            &mut self.buffers
        } else {
            &mut self.results
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
        self.marked_results = HashSet::new();
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
        self.collection().iter()
    }

    fn selection(&self) -> Option<&DisplayablePath> {
        self.collection().selection()
    }

    fn selected_index(&self) -> usize {
        self.collection().selected_index()
    }

    fn select_previous(&mut self) {
        self.collection_mut().select_previous();
    }

    fn select_next(&mut self) {
        self.collection_mut().select_next();
    }

    fn config(&self) -> &SearchSelectConfig {
        &self.config
    }

    fn message(&mut self) -> Option<String> {
        // When multiple buffers are open, we show them instead of query prompts
        if self.buffers.len() > 1 && self.query().is_empty() {
            return None;
        }

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
    use crate::models::application::modes::open::DisplayablePath;
    use crate::models::application::modes::{SearchSelectConfig, SearchSelectMode};
    use crate::models::application::Event;
    use scribe::Workspace;
    use std::env;
    use std::path::Path;
    use std::sync::mpsc::channel;

    #[test]
    fn search_uses_the_query() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
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
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
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

    #[test]
    fn selections_returns_current_selection() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo.toml");
        mode.search();

        let selections: Vec<&DisplayablePath> = mode.selections().iter().copied().collect();
        assert_eq!(selections, vec![mode.results().next().unwrap()]);
    }

    #[test]
    fn selections_includes_marked_selections() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo");
        mode.search();
        mode.toggle_selection();
        mode.select_next();

        let selections: Vec<&DisplayablePath> = mode.selections().iter().copied().collect();
        assert_eq!(selections, mode.results().take(2).collect::<Vec<_>>());
    }

    #[test]
    fn selections_does_not_include_unmarked_indices() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo");
        mode.search();
        mode.toggle_selection();
        mode.toggle_selection();
        mode.select_next();

        let selections: Vec<&DisplayablePath> = mode.selections().iter().copied().collect();
        assert_eq!(selections, vec![mode.results().nth(1).unwrap()]);
    }

    #[test]
    fn selected_indices_returns_current_index() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo.toml");
        mode.search();

        assert_eq!(mode.selected_indices(), vec![0]);
    }

    #[test]
    fn selected_indices_includes_marked_indices() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo");
        mode.search();
        mode.toggle_selection();
        mode.select_next();

        assert_eq!(mode.selected_indices(), vec![0, 1]);
    }

    #[test]
    fn selected_indices_does_not_include_unmarked_indices() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        mode.query().push_str("Cargo");
        mode.search();
        mode.toggle_selection();
        mode.toggle_selection();
        mode.select_next();

        assert_eq!(mode.selected_indices(), vec![1]);
    }

    #[test]
    fn search_clears_marked_indices() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender.clone(), config.clone())
            .unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        // Produce results and mark one of them
        mode.query().push_str("Cargo");
        mode.search();
        mode.toggle_selection();

        // Change the search results
        mode.query().push_str(".");
        mode.search();

        // Ensure the previously-marked result isn't currently selected
        mode.select_next();

        // Verify that the marked result isn't included
        assert_eq!(mode.selected_indices(), vec![1]);
    }

    #[test]
    fn reset_clears_marked_indices() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, receiver) = channel();

        // Populate the index
        mode.reset(&mut workspace, None, sender.clone(), config.clone())
            .unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }

        // Produce results and mark one of them
        mode.query().push_str("Cargo");
        mode.search();
        mode.toggle_selection();

        // Reset the mode and repopulate the index
        mode.reset(&mut workspace, None, sender, config).unwrap();
        if let Ok(Event::OpenModeIndexComplete(index)) = receiver.recv() {
            mode.set_index(index);
        }
        mode.query().push_str("Cargo");
        mode.search();

        // Ensure the previously-marked result isn't currently selected
        mode.select_next();

        // Verify that the marked result isn't included
        assert_eq!(mode.selected_indices(), vec![1]);
    }

    #[test]
    fn when_query_is_empty_results_are_open_buffers() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, _) = channel();

        // Open buffers
        let path1 = Path::new("src/main.rs");
        let path2 = Path::new("src/lib.rs");
        workspace.open_buffer(&path1).unwrap();
        workspace.open_buffer(&path2).unwrap();

        // Let the mode look-up the buffers
        mode.reset(&mut workspace, None, sender.clone(), config.clone())
            .unwrap();

        assert_eq!(
            mode.results().collect::<Vec<_>>(),
            vec![
                &DisplayablePath(path1.into()),
                &DisplayablePath(path2.into())
            ]
        );
    }

    #[test]
    fn open_buffers_respects_workspace_current_buffer() {
        let path = env::current_dir().expect("can't get current directory/path");
        let mut workspace = Workspace::new(&path, None).unwrap();
        let config = SearchSelectConfig::default();
        let mut mode = OpenMode::new(path.clone(), config.clone());
        let (sender, _) = channel();

        // Open buffers
        let path1 = Path::new("src/main.rs");
        let path2 = Path::new("src/lib.rs");
        workspace.open_buffer(&path1).unwrap();
        workspace.open_buffer(&path2).unwrap();

        // Let the mode look-up the buffers
        mode.reset(&mut workspace, None, sender.clone(), config.clone())
            .unwrap();

        assert_eq!(mode.selection(), Some(&DisplayablePath(path2.into())));
    }
}
