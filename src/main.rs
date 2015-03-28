#![feature(convert)]

extern crate scribe;
extern crate rustbox;
extern crate pad;

use std::env;
use view::Mode;
use std::path::PathBuf;

mod view;

fn main() {
    // Set up a workspace in the current directory.
    let mut workspace = match env::current_dir() {
        Ok(path) => scribe::workspace::new(path),
        Err(error) => panic!("Could not initialize workspace to the current directory."),
    };

    // Try to open the specified file.
    // TODO: Handle non-existent files as new empty buffers.
    for path in env::args() {
        let mut argument_buffer = scribe::buffer::from_file(PathBuf::from(path)).unwrap();
        workspace.add_buffer(argument_buffer);
    }

    let mut view = view::new();
    let mut jump_input = String::new();
    let mut tokens = workspace.current_buffer().unwrap().tokens();

    // Set the view's initial status line.
    match workspace.current_buffer().unwrap().file_name() {
        Some(file_name) => view.status_line = file_name,
        None => (),
    }

    loop {
        // Refresh the text and cursor on-screen.
        view.set_cursor(&*workspace.current_buffer().unwrap().cursor);
        view.display(&tokens);

        match view.get_input() {
            Some(c) => {
                match view.mode {
                    Mode::Insert => {
                        if c == '\\' {
                            view.mode = Mode::Normal;
                        } else {
                            let mut buffer = workspace.current_buffer().unwrap();
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
                            tokens = buffer.tokens(); 
                        }
                    },
                    Mode::Normal => {
                        if c == '\t' {
                            workspace.next_buffer();
                        } else {
                            let mut buffer = workspace.current_buffer().unwrap();
                            match c {
                                'q' => break,
                                'j' => {
                                    buffer.cursor.move_down();
                                },
                                'k' => {
                                    buffer.cursor.move_up();
                                },
                                'h' => {
                                    buffer.cursor.move_left();
                                },
                                'l' => {
                                    buffer.cursor.move_right();
                                },
                                'x' => {
                                    buffer.delete();
                                },
                                'i' => {
                                    view.mode = Mode::Insert;
                                },
                                's' => {
                                    buffer.save();
                                },
                                'H' => {
                                    buffer.cursor.move_to_start_of_line();
                                },
                                'L' => {
                                    buffer.cursor.move_to_end_of_line();
                                },
                                'f' => {
                                    view.mode = Mode::Jump;
                                    jump_input = String::new();
                                },
                                _ => continue,
                            }
                        }
                    },
                    Mode::Jump => {
                        if c == '\\' {
                            view.mode = Mode::Normal;
                        } else {
                            // Add the input to whatever we've received in jump mode so far.
                            jump_input.push(c);

                            match jump_input.len() {
                                0 | 1 => (), // Not enough data to match to a position.
                                _ => { 
                                    // Try to find a position, falling back
                                    // to normal mode whether or not we find one.
                                    match view.jump_tag_positions.get(&jump_input) {
                                        Some(position) => {
                                            workspace.current_buffer().unwrap().cursor.move_to(position.clone());
                                        }
                                        None => (),
                                    }

                                    // All done here.
                                    view.mode = Mode::Normal;
                                },
                            }
                        }
                    },
                }
            },
            None => {},
        }
    }
}
