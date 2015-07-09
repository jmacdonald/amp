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
        Mode::Open(ref open_mode) => {
        },
        _ => (),
    }
}
