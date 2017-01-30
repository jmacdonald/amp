use errors::*;
use commands::Result;
use models::application::Application;

pub fn scroll_up(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_up(buffer, 10);
    Ok(())
}

pub fn scroll_down(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_down(buffer, 10);
    Ok(())
}

pub fn scroll_to_cursor(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_to_cursor(buffer);
    Ok(())
}

pub fn scroll_cursor_to_center(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_to_center(buffer);
    Ok(())
}
