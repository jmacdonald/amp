extern crate scribe;
extern crate rustbox;
extern crate pad;

use std::os;

mod view;

enum Mode {
    Normal,
    Insert,
}

fn main() {
    // Set up a workspace in the current directory.
    let mut workspace = match os::getcwd() {
        Ok(path) => scribe::workspace::new(path),
        Err(error) => panic!("Could not initialize workspace to the current directory."),
    };

    // Try to open the specified file.
    // TODO: Handle non-existent files as new empty buffers.
    let path = os::args()[1].clone();
    let mut argument_buffer = scribe::buffer::from_file(Path::new(path)).unwrap();
    workspace.add_buffer(argument_buffer);

    let view = view::new();
    let mut mode = Mode::Normal;

    loop {
        // Refresh the text and cursor on-screen.
        view.display(workspace.current_buffer().unwrap().tokens());
        view.set_cursor(&*workspace.current_buffer().unwrap().cursor);

        // Print the buffer's filename to the status bar, if available.
        match workspace.current_buffer().unwrap().filename() {
            Some(filename) => view.display_status_bar(filename.as_slice()),
            None => (),
        }

        match view.get_input() {
            Some(c) => {
                match mode {
                    Mode::Insert => {
                        if c == '\\' {
                            mode = Mode::Normal;
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
                                buffer.insert(c.to_string().as_slice());
                                if c == '\n' {
                                    buffer.cursor.move_down();
                                    buffer.cursor.move_to_start_of_line();
                                } else {
                                    buffer.cursor.move_right();
                                }
                            }
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
                                    mode = Mode::Insert;
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
                                _ => continue,
                            }
                        }
                    }
                }
            },
            None => {},
        }
    }
}
