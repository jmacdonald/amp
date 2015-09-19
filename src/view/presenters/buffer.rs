extern crate scribe;
extern crate rustbox;

use view::{Data, StatusLine, scrollable_region};
use models::application::Mode;
use models::application::modes::insert::InsertMode;
use models::application::modes::jump::JumpMode;
use models::application::modes::open::OpenMode;
use models::application::modes::select::SelectMode;
use models::terminal::Terminal;
use scribe::buffer::{Buffer, Position};
use rustbox::Color;

pub struct BufferPresenter {
    region: scrollable_region::ScrollableRegion,
}

impl BufferPresenter {
    pub fn data(&mut self, buffer: &mut Buffer, mode: &mut Mode<InsertMode, JumpMode, OpenMode, SelectMode>) -> Data {
        // Update the visible buffer range to include the cursor, if necessary.
        self.region.scroll_into_view(buffer.cursor.line);

        // Build status line data.
        let content = match buffer.path {
            Some(ref path) => path.to_string_lossy().into_owned(),
            None => String::new(),
        };
        let color = match mode {
            &mut Mode::Insert(_) => { Color::Green },
            _ => { Color::Black }
        };

        // Get the buffer's tokens, transforming them if we're in jump mode.
        let tokens = match mode {
            &mut Mode::Jump(ref mut jump_mode) => {
                jump_mode.tokens(
                    &buffer.tokens(),
                    Some(self.region.visible_range())
                )
            },
            _ => buffer.tokens(),
        };

        // The buffer tracks its cursor absolutely, but the
        // view must display it relative to any scrolling.
        let relative_cursor = Position{
            line: self.region.relative_position(buffer.cursor.line),
            offset: buffer.cursor.offset
        };

        // Bundle up the presentable data.
        Data{
            tokens: tokens,
            visible_range: self.region.visible_range(),
            cursor: relative_cursor,
            status_line: StatusLine{
                content: content,
                color: color
            }
        }
    }
}

pub fn new(terminal: &Terminal) -> BufferPresenter {
    let region = scrollable_region::new(terminal.height()-2);
    BufferPresenter{ region: region }
}
