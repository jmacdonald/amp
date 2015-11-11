use std::mem;
use models::application::modes::jump;
use models::application::{Mode, Application};

pub fn match_tag(app: &mut Application) {
    let done = match app.mode {
        Mode::Jump(ref mut jump_mode) => {
            match jump_mode.input.len() {
                0 | 1 => false, // Not enough data to match to a position.
                _ => {
                    // Try to find a position, falling back
                    // to normal mode whether or not we find one.
                    match jump_mode.map_tag(&jump_mode.input) {
                        Some(position) => {
                            match app.workspace.current_buffer() {
                                Some(buffer) => buffer.cursor.move_to(position.clone()),
                                None => false,
                            };
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
        // Swap out the application's jump mode.
        let old_mode = mem::replace(&mut app.mode, Mode::Normal);

        // Now that we own the jump mode, switch to
        // the previous select mode, if there was one.
        match old_mode {
            Mode::Jump(jump_mode) => {
                match jump_mode.select_mode {
                    jump::SelectModeOptions::None => (),
                    jump::SelectModeOptions::Select(select_mode) => {
                        app.mode = Mode::Select(select_mode);
                    },
                    jump::SelectModeOptions::SelectLine(select_mode) => {
                        app.mode = Mode::SelectLine(select_mode);
                    },
                }
            }
            _ => (),
        }
    }
}
