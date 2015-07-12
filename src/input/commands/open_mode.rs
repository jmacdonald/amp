extern crate bloodhound;
extern crate scribe;

use application::Mode;
use application::Application;
use input::commands;

pub fn open(app: &mut Application) {
    match app.mode {
        Mode::Open(ref mut mode) => {
            match mode.results {
                Some(ref results) => {
                    let ref selection = results[mode.selected_result_index];
                    match scribe::buffer::from_file(selection.path.clone()) {
                        Ok(buffer) => {
                            app.workspace.add_buffer(buffer);
                        },
                        Err(_) => (),
                    }
                },
                None => (),
            }
        },
        _ => (),
    }

    // FIXME: This should be moved into Ok match result, but we cannot
    // lend out a reference to app once we've matched against its mode.
    commands::application::switch_to_normal_mode(app);
}

pub fn search(app: &mut Application) {
    match app.mode {
        Mode::Open(ref mut mode) => {
            mode.results = Some(
                bloodhound::matching::find(
                    &mode.input,         // The query string (needle).
                    &mode.index.entries, // The indexed files (haystack).
                    5                    // The max number of results.
                )
            );
        },
        _ => (),
    }
}
