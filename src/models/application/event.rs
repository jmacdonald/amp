use input::Key;

#[derive(Debug, PartialEq)]
pub enum Event {
    Key(Key)
}
