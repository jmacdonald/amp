extern crate scribe;
extern crate rustbox;

pub mod modes;

use std::env;
use std::path::PathBuf;
use self::modes::jump::JumpMode;
use self::modes::insert::InsertMode;
use self::modes::open::OpenMode;
use self::modes::select::SelectMode;
use self::modes::select_line::SelectLineMode;
use self::modes::search_insert::SearchInsertMode;
use scribe::workspace::Workspace;
use view::buffer::BufferView;

pub enum Mode {
    Normal,
    Insert(InsertMode),
    Jump(JumpMode),
    Open(OpenMode),
    Select(SelectMode),
    SelectLine(SelectLineMode),
    SearchInsert(SearchInsertMode),
    Exit,
}

pub enum Clipboard {
    Inline(String),
    Block(String),
    None,
}

pub struct Application {
    pub mode: Mode,
    pub workspace: Workspace,
    pub clipboard: Clipboard,
    pub search_query: Option<String>,
    pub buffer_view: BufferView,
}

pub fn new(buffer_height: usize) -> Application {
    // Set up a workspace in the current directory.
    let mut workspace = match env::current_dir() {
        Ok(path) => scribe::workspace::new(path),
        Err(_) => panic!("Could not initialize workspace to the current directory."),
    };

    // Try to open the specified file.
    // TODO: Handle non-existent files as new empty buffers.
    for path in env::args().skip(1) {
        let argument_buffer = match scribe::buffer::from_file(PathBuf::from(path.clone())) {
            Ok(buf) => buf,
            Err(_) => panic!("Ran into an error trying to open {}.", path),
        };

        workspace.add_buffer(argument_buffer);
    }

    let buffer_view = ::view::buffer::new(buffer_height);

    Application{
        mode: Mode::Normal,
        workspace: workspace,
        clipboard: Clipboard::None,
        search_query: None,
        buffer_view: buffer_view,
    }
}
