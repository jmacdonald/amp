extern crate scribe;
extern crate rustbox;

pub mod modes;

use std::env;
use std::path::PathBuf;
use self::modes::jump::JumpMode;
use self::modes::insert::InsertMode;
use self::modes::open::OpenMode;
use self::modes::select::SelectMode;
use scribe::workspace::Workspace;

#[derive(PartialEq)]
pub enum Mode<I, J, O, S> {
    Normal,
    Insert(I),
    Jump(J),
    Open(O),
    Select(S),
    Exit,
}

pub struct Application {
    pub mode: Mode<InsertMode, JumpMode, OpenMode, SelectMode>,
    pub workspace: Workspace,
}

pub fn new() -> Application {
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

    Application{ mode: Mode::Normal, workspace: workspace }
}
