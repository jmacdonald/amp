use crate::input::Key;
use crate::models::application::modes::open::Index;

#[derive(Debug, PartialEq)]
pub enum Event {
    Key(Key),
    Resize,
    OpenModeIndexComplete(Index)
}
