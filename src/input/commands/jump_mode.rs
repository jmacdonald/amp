use application::Mode;
use application::Application;
use application::jump_mode;

pub fn match_tag(app: &mut Application, data: &char) {
    let done = match app.mode {
        Mode::Jump(ref mut jump_mode) => {
            match jump_mode.input.len() {
                0 | 1 => false, // Not enough data to match to a position.
                _ => { 
                    // Try to find a position, falling back
                    // to normal mode whether or not we find one.
                    match jump_mode.map_tag(&jump_mode.input) {
                        Some(position) => {
                            app.workspace.current_buffer().
                                unwrap().cursor.move_to(position.clone());
                        }
                        None => (),
                    }

                    // All done here.
                    true
                },
            }
        },
        _ => false,
    };

    if done {
        app.mode = Mode::Normal
    }
}
