use crate::errors::*;
use crate::presenters::current_buffer_status_line_data;
use scribe::Workspace;
use crate::models::application::modes::JumpMode;
use crate::view::{Colors, StatusLineData, Style, Terminal, View};

pub fn display(workspace: &mut Workspace, view: &mut View, error: &Error) {
    let mut presenter = view.build_presenter().unwrap();

    let mut data = String::new();
    if let Some(buffer) = workspace.current_buffer() {
        data = buffer.data();
        presenter.draw_buffer(buffer, &data, None, None);
    }

    presenter.print_status_line(&[StatusLineData {
        content: error.description().to_string(),
        style: Style::Bold,
        colors: Colors::Warning,
    }]);

    presenter.present();
}
