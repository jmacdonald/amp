#![feature(convert)]

extern crate scribe;
extern crate rustbox;
extern crate pad;

use application::Mode;

mod application;
mod terminal;
mod view;
mod input;

fn main() {
    let mut application = application::new();
    let terminal = terminal::new();
    let mut view = view::new(&terminal);
    let mut jump_input = String::new();

    // Set the view's initial status line.
    match application.workspace.current_buffer().unwrap().file_name() {
        Some(file_name) => view.status_line = file_name,
        None => (),
    }

    loop {
        // Refresh the text and cursor on-screen.
        view.set_cursor(&terminal, &*application.workspace.current_buffer().unwrap().cursor);
        match application.mode {
            Mode::Jump => {
                // Transform the buffer tokens before displaying them.
                let jump_tokens = view.jump_mode.tokens(&application.workspace.current_buffer().unwrap().tokens());

                view.display(&terminal, &jump_tokens);
            },
            _ => {
                view.display(&terminal, &application.workspace.current_buffer().unwrap().tokens());
            },
        }

        match terminal.get_input() {
            Some(c) => {
                match application.mode {
                    Mode::Insert => {
                        if c == '\\' {
                            application.mode = Mode::Normal;
                        } else {
                            let mut buffer = application.workspace.current_buffer().unwrap();
                            if c == '\u{8}' || c == '\u{127}' {
                                if buffer.cursor.offset == 0 {
                                    buffer.cursor.move_up();
                                    buffer.cursor.move_to_end_of_line();
                                    buffer.delete();
                                } else {
                                    buffer.cursor.move_left();
                                    buffer.delete();
                                }
                            } else {
                                buffer.insert(&c.to_string());
                                if c == '\n' {
                                    buffer.cursor.move_down();
                                    buffer.cursor.move_to_start_of_line();
                                } else {
                                    buffer.cursor.move_right();
                                }
                            }
                        }
                    },
                    Mode::Normal => input::modes::normal::handle(c).execute(&mut application),
                    Mode::Jump => {
                        if c == '\\' {
                            application.mode = Mode::Normal;
                        } else {
                            // Add the input to whatever we've received in jump mode so far.
                            jump_input.push(c);

                            match jump_input.len() {
                                0 | 1 => (), // Not enough data to match to a position.
                                _ => { 
                                    // Try to find a position, falling back
                                    // to normal mode whether or not we find one.
                                    match view.jump_mode.map_tag(&jump_input) {
                                        Some(position) => {
                                            application.workspace.current_buffer().unwrap().cursor.move_to(position.clone());
                                        }
                                        None => (),
                                    }

                                    // All done here.
                                    application.mode = Mode::Normal;
                                },
                            }
                        }
                    },
                    Mode::Exit => break,
                }
            },
            None => {},
        }
    }
}
