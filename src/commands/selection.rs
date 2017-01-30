use models::application::{Application, ClipboardContent, Mode};
use scribe::buffer::{LineRange, Range};
use super::application;
use errors::*;
use commands::{self, Result};
use helpers;

pub fn delete(app: &mut Application) -> Result {
    if let Some(buffer) = app.workspace.current_buffer() {
        match app.mode {
            Mode::Select(ref select_mode) => {
                let cursor_position = *buffer.cursor.clone();
                let delete_range = Range::new(cursor_position, select_mode.anchor);
                buffer.delete_range(delete_range.clone());
                buffer.cursor.move_to(delete_range.start());
            }
            Mode::SelectLine(ref mode) => {
                let delete_range = mode.to_range(&*buffer.cursor);
                buffer.delete_range(delete_range.clone());
                buffer.cursor.move_to(delete_range.start());
            }
            _ => bail!("Can't delete selections outside of select mode"),
        };
    } else {
        bail!(BUFFER_MISSING);
    }

    application::switch_to_normal_mode(app)?;
    commands::view::scroll_to_cursor(app)
}

pub fn copy_and_delete(app: &mut Application) -> Result {
    let _ = copy_to_clipboard(app);
    delete(app)
}

pub fn change(app: &mut Application) -> Result {
    let _ = copy_to_clipboard(app);
    delete(app)?;
    application::switch_to_insert_mode(app)
}

pub fn copy(app: &mut Application) -> Result {
    copy_to_clipboard(app)?;
    application::switch_to_normal_mode(app)?;
    commands::view::scroll_to_cursor(app)
}

fn copy_to_clipboard(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

    match app.mode {
        Mode::Select(ref select_mode) => {
            let cursor_position = *buffer.cursor.clone();
            let selected_range = Range::new(cursor_position, select_mode.anchor);

            let data = buffer.read(&selected_range.clone())
                .ok_or("Couldn't read selected data from buffer")?;
            app.clipboard.set_content(ClipboardContent::Inline(data));
        }
        Mode::SelectLine(ref mode) => {
            let selected_range = helpers::inclusive_range(
                &LineRange::new(
                    mode.anchor,
                    buffer.cursor
                    .line
                ),
                buffer
            );

            let data = buffer.read(&selected_range.clone())
                .ok_or("Couldn't read selected data from buffer")?;
            app.clipboard.set_content(ClipboardContent::Block(data));
        }
        _ => bail!("Can't copy data to clipboard outside of select modes"),
    };

    Ok(())
}
