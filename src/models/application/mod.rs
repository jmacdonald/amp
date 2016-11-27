extern crate git2;
extern crate scribe;

pub mod modes;
mod clipboard;

// Published API
pub use self::clipboard::ClipboardContent;

use std::env;
use std::path::PathBuf;
use self::modes::{JumpMode, LineJumpMode, SymbolJumpMode, InsertMode, OpenMode, SelectMode, SelectLineMode, SearchInsertMode};
use scribe::{Buffer, Workspace};
use view::View;
use self::clipboard::Clipboard;
use self::git2::Repository;

pub enum Mode {
    Normal,
    Insert(InsertMode),
    Jump(JumpMode),
    LineJump(LineJumpMode),
    SymbolJump(SymbolJumpMode),
    Open(OpenMode),
    Select(SelectMode),
    SelectLine(SelectLineMode),
    SearchInsert(SearchInsertMode),
    Exit,
}

pub struct Application {
    pub mode: Mode,
    pub workspace: Workspace,
    pub search_query: Option<String>,
    pub view: View,
    pub clipboard: Clipboard,
    pub repository: Option<Repository>,
}

pub fn new() -> Application {
    // Set up a workspace in the current directory.
    let mut workspace = match env::current_dir() {
        Ok(path) => Workspace::new(path),
        Err(_) => panic!("Could not initialize workspace to the current directory."),
    };

    // Try to open the specified file.
    // TODO: Handle non-existent files as new empty buffers.
    for path in env::args().skip(1) {
        let argument_buffer = match Buffer::from_file(PathBuf::from(path.clone())) {
            Ok(buf) => buf,
            Err(_) => panic!("Ran into an error trying to open {}.", path),
        };

        workspace.add_buffer(argument_buffer);
    }

    let view = View::new();
    let clipboard = Clipboard::new();

    Application {
        mode: Mode::Normal,
        workspace: workspace,
        search_query: None,
        view: view,
        clipboard: clipboard,
        repository: find_repo(),
    }
}

// Searches upwards for a repository, starting from the current directory.
fn find_repo() -> Option<Repository> {
    let initial_path = env::current_dir().unwrap();
    let mut current_path = Some(initial_path.as_path());

    while let Some(path) = current_path {
        if let Ok(repo) = Repository::open(path) {
            return Some(repo);
        } else {
            current_path = path.parent();
        }
    }

    None
}
