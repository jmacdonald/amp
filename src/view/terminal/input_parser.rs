use crate::input::Key;
use crate::models::application::Event;

pub struct InputParser {
    data: Vec<u8>,
    offset: usize,
}

impl InputParser {
    pub fn new() -> InputParser {
        InputParser {
            data: Vec::new(),
            offset: 0,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
}

impl Iterator for InputParser {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            self.data.clear();
            self.offset = 0;
            return None;
        }

        let slice = &self.data[self.offset..];

        let (key, consumed) = match slice {
            [0x1B, b'[', b'A', ..] => (Key::Up, 3),
            [0x1B, b'[', b'B', ..] => (Key::Down, 3),
            [0x1B, b'[', b'C', ..] => (Key::Right, 3),
            [0x1B, b'[', b'D', ..] => (Key::Left, 3),
            [0x1B, b'[', b'H', ..] => (Key::Home, 3),
            [0x1B, b'[', b'F', ..] => (Key::End, 3),
            [0x1B, b'[', b'2', b'~', ..] => (Key::Insert, 4),
            [0x1B, b'[', b'3', b'~', ..] => (Key::Delete, 4),
            [0x1B, b'[', b'5', b'~', ..] => (Key::PageUp, 4),
            [0x1B, b'[', b'6', b'~', ..] => (Key::PageDown, 4),
            [0x1B, ..] => (Key::Esc, 1),
            [0x7F, ..] | [0x08, ..] => (Key::Backspace, 1),
            [0x0A, ..] | [0x0D, ..] => (Key::Enter, 1),
            [0x09, ..] => (Key::Tab, 1),
            [b @ 0x01..=0x1A, ..] => (Key::Ctrl((b + b'a' - 1) as char), 1),
            [b @ 0x20..=0x7E, ..] => (Key::Char(*b as char), 1),
            _ => return None,
        };

        self.offset += consumed;

        Some(Event::Key(key))
    }
}
