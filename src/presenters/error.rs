use crate::errors::*;
use crate::view::{Colors, StatusLineData, Style, View};
use scribe::Workspace;

pub fn display(workspace: &mut Workspace, view: &mut View, error: &Error) -> Result<()> {
    let data;
    let mut presenter = view.build_presenter().unwrap();

    if let Some(buffer) = workspace.current_buffer.as_ref() {
        data = buffer.data();
        let _ = presenter.print_buffer(buffer, &data, &workspace.syntax_set, None, None);
    }

    presenter.print_status_line(&[StatusLineData {
        content: error.description().to_string(),
        style: Style::Bold,
        colors: Colors::Warning,
    }]);

    presenter.present()?;

    Ok(())
}
