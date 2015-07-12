extern crate bloodhound;

use application::Mode;
use application::Application;

pub fn open(app: &mut Application) {
    match app.mode {
        Mode::Open(ref open_mode) => {
        },
        _ => (),
    }
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
