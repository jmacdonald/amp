extern crate scribe;
extern crate rustbox;

use std::os;

mod view;

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
    let mut buffer = workspace.current_buffer().unwrap();
    view.display(buffer.tokens());
    view.set_cursor(&*buffer.cursor);
    let mut insert = false;

    loop {
        match view.get_input() {
            Some(c) => {
                if insert {
                    if c == '\\' {
                        insert = false
                    } else {
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
                        view.set_cursor(&*buffer.cursor);
                        view.display(buffer.tokens());
                    }
                } else {
                    match c {
                        'q' => break,
                        'j' => {
                            buffer.cursor.move_down();
                            view.set_cursor(&*buffer.cursor);
                        },
                        'k' => {
                            buffer.cursor.move_up();
                            view.set_cursor(&*buffer.cursor);
                        },
                        'h' => {
                            buffer.cursor.move_left();
                            view.set_cursor(&*buffer.cursor);
                        },
                        'l' => {
                            buffer.cursor.move_right();
                            view.set_cursor(&*buffer.cursor);
                        },
                        'x' => {
                            buffer.delete();
                            view.display(buffer.tokens());
                            view.set_cursor(&*buffer.cursor);
                        },
                        'i' => {
                            insert = true;
                        },
                        's' => {
                            buffer.save();
                        },
                        'H' => {
                            buffer.cursor.move_to_start_of_line();
                            view.set_cursor(&*buffer.cursor);
                        },
                        'L' => {
                            buffer.cursor.move_to_end_of_line();
                            view.set_cursor(&*buffer.cursor);
                        },
                        _ => continue,
                    }
                }
            },
            None => {},
        }
    }
}
