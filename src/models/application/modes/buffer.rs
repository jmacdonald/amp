use std::slice::Iter;
use std::fmt;
use std::path::PathBuf;
use crate::util::SelectableVec;
use scribe::Workspace;
use fragment;
use crate::models::application::modes::{SearchSelectMode, SearchSelectConfig};

#[derive(Clone)]
pub struct BufferEntry {
    pub id: usize,
    pub path: Option<PathBuf>,
    pub is_modified: bool,
    search_str: String,
}

impl fragment::matching::AsStr for BufferEntry {
    fn as_str(&self) -> &str {
        &self.search_str
    }
}

impl ::std::fmt::Display for BufferEntry {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let state;
        if self.is_modified {
            state = "* ";
        } else {
            state = "  ";
        }
        write!(f, "{}#{:<3} {}", state, self.id, self.search_str)
    }
}

pub struct BufferMode {
    insert: bool,
    empty_msg: String,
    input: String,
    buffers: Vec<BufferEntry>,
    results: SelectableVec<BufferEntry>,
    config: SearchSelectConfig,
}

impl BufferMode {
    pub fn new(workspace: &mut Workspace, config: SearchSelectConfig) -> BufferMode {
        // ToDo: This code assumes the id is _ALWAYS_ valid in a workspace
        let buffers: Vec<_> = workspace.iter_buffers().map(|buffer| {
            let id = buffer.id.unwrap();
            let path = buffer.get_path().map(|p| PathBuf::from(p));
            let search_str = path.as_ref().map(|p| p.to_string_lossy().into())
                .unwrap_or_else(|| "<not named>".into());
            let is_modified = buffer.modified();
            BufferEntry { id, path, is_modified, search_str }
        }).collect();

        BufferMode {
            insert: true,
            empty_msg: "No buffers are open.".into(),
            input: String::new(),
            buffers,
            results: SelectableVec::new(Vec::new()),
            config,
        }
    }

    pub fn apply_filter<F: FnMut(&BufferEntry) -> bool>(&mut self, msg: String, mut f: F) {
        // Note: It would be more perfomant to perfom the filtering during new, but
        // it seems unlikely it matters in this case.
        let mut buffers = Vec::new();
        ::std::mem::swap(&mut buffers, &mut self.buffers);
        self.buffers.extend(buffers.into_iter().filter(|e| f(e)));
        self.empty_msg = msg;
    }
}

impl fmt::Display for BufferMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BUFFER")
    }
}

impl SearchSelectMode<BufferEntry> for BufferMode {
    fn search(&mut self) {
        let results: Vec<_> = if self.input.is_empty() {
            self.buffers
                .iter()
                .take(self.config.max_results)
                .map(|r| r.clone())
                .collect()
        } else {
            fragment::matching::find(
                &self.input,
                &self.buffers,
                self.config.max_results
            ).into_iter().map(|r| r.clone()).collect()
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

    fn results(&self) -> Iter<BufferEntry> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&BufferEntry> {
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
        if !self.results.is_empty() {
            None
        } else if self.input.is_empty() {
            Some(self.empty_msg.clone())
        } else {
            Some(String::from("No matching entries found."))
        }
    }
}
