use crate::errors::*;
use crate::commands::Result;
use crate::models::application::Application;
use crate::view::Terminal;

pub fn scroll_up<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_up(buffer, 10)?;
    Ok(())
}

pub fn scroll_down<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_down(buffer, 10)?;
    Ok(())
}

pub fn scroll_to_cursor<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_to_cursor(buffer)?;
    Ok(())
}

pub fn scroll_cursor_to_center<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    app.view.scroll_to_center(buffer)?;
    Ok(())
}
