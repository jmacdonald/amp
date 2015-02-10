extern crate scribe;
extern crate rustbox;

mod view;

fn main() {
    let mut buffer = scribe::buffer::from_file(&Path::new("./Cargo.toml")).unwrap();
    let view = view::new();
    view.display(buffer.data().as_slice());

    loop {
        match view.get_input() {
            Some('q') => break,
            Some('j') => {
                let position = scribe::buffer::Position{ line: buffer.cursor.line+1, offset: buffer.cursor.offset };
                buffer.move_cursor(position);
                view.set_cursor(&*buffer.cursor);
            },
            Some('k') => {
                let position = scribe::buffer::Position{ line: buffer.cursor.line-1, offset: buffer.cursor.offset };
                buffer.move_cursor(position);
                view.set_cursor(&*buffer.cursor);
            },
            Some('h') => {
                let position = scribe::buffer::Position{ line: buffer.cursor.line, offset: buffer.cursor.offset-1 };
                buffer.move_cursor(position);
                view.set_cursor(&*buffer.cursor);
            },
            Some('l') => {
                let position = scribe::buffer::Position{ line: buffer.cursor.line, offset: buffer.cursor.offset+1 };
                buffer.move_cursor(position);
                view.set_cursor(&*buffer.cursor);
            },
            Some(_) => {},
            None => {},
        }
    }
}
