extern crate scribe;
extern crate rustbox;

use std::env;
use std::path::PathBuf;
use scribe::workspace::Workspace;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Jump,
    Exit,
}

pub struct Application {
    pub mode: Mode,
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
