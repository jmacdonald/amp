use input::Key;
use models::application::modes::open::Index;

#[derive(Debug, PartialEq)]
pub enum Event {
    Key(Key),
    Resize,
    OpenModeIndexComplete(Index)
}
