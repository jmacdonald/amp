extern crate git2;
extern crate scribe;

pub mod modes;
mod clipboard;

// Published API
pub use self::clipboard::ClipboardContent;

use std::env;
use std::path::Path;
use std::io::Result;
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

impl Application {
    pub fn new() -> Result<Application> {
        let current_dir = try!(env::current_dir());

        // Set up a workspace in the current directory.
        let mut workspace = try!(Workspace::new(&current_dir));

        // Try to open the specified file.
        // TODO: Handle non-existent files as new empty buffers.
        for path_arg in env::args().skip(1) {
            let argument_buffer = match Buffer::from_file(Path::new(&path_arg)) {
                Ok(buf) => buf,
                Err(_) => panic!("Ran into an error trying to open {}.", path_arg),
            };

            workspace.add_buffer(argument_buffer);
        }

        let view = View::new();
        let clipboard = Clipboard::new();

        Ok(Application {
            mode: Mode::Normal,
            workspace: workspace,
            search_query: None,
            view: view,
            clipboard: clipboard,
            repository: Repository::discover(&current_dir).ok(),
        })
    }
}
