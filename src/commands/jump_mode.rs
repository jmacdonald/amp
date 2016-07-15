use std::mem;
use models::application::modes::jump;
use models::application::modes::JumpMode;
use models::application::{Mode, Application};
use scribe::Workspace;

pub fn match_tag(app: &mut Application) {
    if let Mode::Jump(ref mut jump_mode) = app.mode {
        match jump_mode.input.len() {
            0 => return, // Not enough data to match to a position.
            1 => {
                if jump_mode.line_mode {
                    jump_to_tag(jump_mode, &mut app.workspace);
                } else {
                    return // Not enough data to match to a position.
                }
            },
            _ => jump_to_tag(jump_mode, &mut app.workspace),
        }
    };

    switch_to_previous_mode(app);
}

// Try to find a position for the input tag and jump to it.
fn jump_to_tag(jump_mode: &mut JumpMode, workspace: &mut Workspace) {
    if let Some(position) = jump_mode.map_tag(&jump_mode.input) {
        if let Some(buf) = workspace.current_buffer(){
            buf.cursor.move_to(position.clone());
        };
    }
}

fn switch_to_previous_mode(app: &mut Application) {
    let old_mode = mem::replace(&mut app.mode, Mode::Normal);

    // Now that we own the jump mode, switch to
    // the previous select mode, if there was one.
    match old_mode {
        Mode::Jump(jump_mode) => {
            match jump_mode.select_mode {
                jump::SelectModeOptions::None => (),
                jump::SelectModeOptions::Select(select_mode) => {
                    app.mode = Mode::Select(select_mode);
                }
                jump::SelectModeOptions::SelectLine(select_mode) => {
                    app.mode = Mode::SelectLine(select_mode);
                }
            }
        }
        _ => (),
    }
}
