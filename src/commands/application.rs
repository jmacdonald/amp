use commands;
use models::application::modes::jump::JumpMode;
use models::application::modes::insert::InsertMode;
use models::application::modes::open::OpenMode;
use models::application::modes::select::SelectMode;
use models::application::modes::search_insert::SearchInsertMode;
use models::application::modes::search_results::SearchResultsMode;
use models::application::{Application, Mode};
use models::application::modes::{insert, jump, open, select, search_insert, search_results};

pub fn switch_to_normal_mode(app: &mut Application) {
    commands::buffer::end_command_group(app);
    app.mode = Mode::Normal;
}
pub fn switch_to_insert_mode(app: &mut Application) {
    commands::buffer::start_command_group(app);
    app.mode = Mode::Insert(insert::new());
}

pub fn switch_to_jump_mode(app: &mut Application) {
    app.mode = Mode::Jump(jump::new());
}

pub fn switch_to_open_mode(app: &mut Application) {
    app.mode = Mode::Open(open::new(app.workspace.path.clone()));
    commands::open_mode::search(app);
}

pub fn switch_to_select_mode(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            app.mode = Mode::Select(select::new(*buffer.cursor.clone()));
        },
        None => (),
    }
}

pub fn switch_to_search_insert_mode(app: &mut Application) {
    if app.workspace.current_buffer().is_some() {
        app.mode = Mode::SearchInsert(search_insert::new());
    }
}

pub fn switch_to_search_results_mode(app: &mut Application) {
    let mode: Option<Mode<InsertMode, JumpMode, OpenMode, SelectMode, SearchInsertMode, SearchResultsMode>> = match app.mode {
        Mode::SearchInsert(ref s) => {
            match app.workspace.current_buffer() {
                Some(buffer) => {
                    Some(
                        Mode::SearchResults(
                            search_results::new(
                                buffer.search(&s.input)
                            )
                        )
                    )
                },
                None => None
            }
        }
        _ => None
    };

    if mode.is_some() {
        app.mode = mode.unwrap()
    }

    commands::search::move_to_current_result(app);
}

pub fn exit(app: &mut Application) {
    app.mode = Mode::Exit;
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use scribe::buffer;
    use scribe::buffer::Position;
    use models::application;
    use models::application::Mode;

    #[test]
    fn switch_to_search_results_mode_populates_results() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedits");
        app.workspace.add_buffer(buffer);

        // Enter search mode and add a search value.
        super::switch_to_search_insert_mode(&mut app);
        match app.mode {
            Mode::SearchInsert(ref mut mode) => mode.input = "ed".to_string(),
            _ => ()
        };

        // Switch to results mode and get the current result.
        super::switch_to_search_results_mode(&mut app);
        let result = match app.mode {
            Mode::SearchResults(mode) => mode.current_result(),
            _ => None
        };
        let expected_position = Position{ line: 0, offset: 4};

        // Ensure the current result is what we're expecting.
        assert_eq!(
            result,
            Some(expected_position)
        );

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            expected_position
        );
    }
}
