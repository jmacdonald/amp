extern crate bloodhound;
extern crate rustbox;
extern crate scribe;

use application::modes::open::OpenMode;
use terminal::Terminal;
use rustbox::Color;
use scribe::buffer::{Position, Token, Category, LineRange};
use pad::PadStr;

pub fn display(terminal: &Terminal, tokens: &Vec<Token>, mode: &OpenMode) {
    terminal.clear();
    for (line, result) in bloodhound::matching::find(&mode.input, &mode.index.entries, 5).iter().enumerate() {
        let color = if line == mode.selected_result_index { Color::Black } else { Color::Default };
        let padded_content = result.path.as_path().to_str().unwrap().pad_to_width(terminal.width());
        terminal.print(0, line, rustbox::RB_NORMAL, Color::White, color, &padded_content);
    }

    // Draw the divider.
    let line = 5;
    let padded_content = mode.input.pad_to_width(terminal.width());
    terminal.print(0, line, rustbox::RB_BOLD, Color::Black, Color::White, &padded_content);

    let mut line = 6;
    let mut offset = 0;
    let line_limit = terminal.height() - 5;
    'print_loop: for token in tokens.iter() {
        let color = match token.category {
            Category::Keyword    => Color::Magenta,
            Category::Identifier => Color::Yellow,
            Category::String     => Color::Red,
            Category::Comment    => Color::Blue,
            Category::Method     => Color::Cyan,
            _                    => Color::Default,
        };

        for character in token.lexeme.chars() {
            if character == '\n' {
                line += 1;
                offset = 0;
            } else if line < line_limit {
                terminal.print_char(offset, line, rustbox::RB_NORMAL, color, Color::Default, character);
                offset += 1;
            }
        }
    }

    terminal.present();
}
