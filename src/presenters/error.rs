use crate::errors::*;
use scribe::Workspace;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View, error: &Error) {
    let data;
    let mut presenter = view.build_presenter().unwrap();

    if let Some(buffer) = workspace.current_buffer() {
        data = buffer.data();
        let _ = presenter.print_buffer(buffer, &data, None, None);
    }

    presenter.print_status_line(&[StatusLineData {
        content: error.description().to_string(),
        style: Style::Bold,
        colors: Colors::Warning,
    }]);

    presenter.present();
}
