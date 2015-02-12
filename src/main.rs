extern crate scribe;
extern crate rustbox;

mod view;

fn main() {
    let mut buffer = scribe::buffer::from_file(&Path::new("./Cargo.toml")).unwrap();
    let view = view::new();
    view.display(buffer.data().as_slice());
    view.set_cursor(&*buffer.cursor);

    loop {
        match view.get_input() {
            Some('q') => break,
            Some('j') => {
                buffer.cursor.move_down();
                view.set_cursor(&*buffer.cursor);
            },
            Some('k') => {
                buffer.cursor.move_up();
                view.set_cursor(&*buffer.cursor);
            },
            Some('h') => {
                buffer.cursor.move_left();
                view.set_cursor(&*buffer.cursor);
            },
            Some('l') => {
                buffer.cursor.move_right();
                view.set_cursor(&*buffer.cursor);
            },
            Some(_) => {},
            None => {},
        }
    }
}
