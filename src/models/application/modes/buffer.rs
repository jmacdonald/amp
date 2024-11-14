use crate::models::application::modes::{SearchSelectConfig, SearchSelectMode};
use crate::util::SelectableVec;
use fragment;
use fragment::matching::AsStr;
use scribe::Workspace;
use std::fmt;
use std::slice::Iter;

#[derive(Clone, Default)]
pub struct WorkspaceBuffer {
    pub title: String,
    pub index: usize,
}

impl fmt::Display for WorkspaceBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl AsStr for WorkspaceBuffer {
    fn as_str(&self) -> &str {
        &self.title
    }
}

#[derive(Default)]
pub struct BufferMode {
    insert: bool,
    input: String,
    buffers: SelectableVec<WorkspaceBuffer>,
    results: SelectableVec<WorkspaceBuffer>,
    config: SearchSelectConfig,
}

impl BufferMode {
    pub fn new(config: SearchSelectConfig) -> BufferMode {
        BufferMode {
            config,
            insert: true,
            ..Default::default()
        }
    }

    pub fn reset(&mut self, workspace: &mut Workspace, config: SearchSelectConfig) {
        *self = BufferMode {
            config,
            insert: true,
            buffers: SelectableVec::new(
                workspace
                    .buffer_paths()
                    .into_iter()
                    .enumerate()
                    .map(|(index, bp)| WorkspaceBuffer {
                        title: bp
                            .map(|p| p.to_string_lossy().into_owned())
                            .unwrap_or(String::from("Untitled")),
                        index,
                    })
                    .collect(),
            ),
            ..Default::default()
        };

        if let Some(i) = workspace.current_buffer_index() {
            self.buffers.set_selected_index(i);
        }
    }
}

impl fmt::Display for BufferMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BUFFER")
    }
}

impl SearchSelectMode for BufferMode {
    type Item = WorkspaceBuffer;

    fn search(&mut self) {
        // Find the buffers we're looking for using the query.
        let results = if self.input.is_empty() {
            self.buffers
                .iter()
                .take(self.config.max_results)
                .cloned()
                .collect()
        } else {
            fragment::matching::find(&self.input, self.buffers.iter(), self.config.max_results)
                .into_iter()
                .map(|i| i.clone())
                .collect()
        };

        self.results = SelectableVec::new(results);

        if self.input.is_empty() {
            self.results
                .set_selected_index(self.buffers.selected_index());
        }
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

    fn results(&self) -> Iter<Self::Item> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&Self::Item> {
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
