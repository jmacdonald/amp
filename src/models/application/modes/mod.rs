mod command;
mod confirm;
pub mod jump;
mod line_jump;
pub mod open;
mod path;
mod search;
mod search_select;
mod select;
mod select_line;
mod symbol_jump;
mod syntax;
mod theme;

pub enum Mode {
    Command(CommandMode),
    Confirm(ConfirmMode),
    Exit,
    Insert,
    Jump(JumpMode),
    LineJump(LineJumpMode),
    Normal,
    Open(OpenMode),
    Paste,
    Path(PathMode),
    Search(SearchMode),
    Select(SelectMode),
    SelectLine(SelectLineMode),
    SymbolJump(SymbolJumpMode),
    Syntax(SyntaxMode),
    Theme(ThemeMode),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ModeKey {
    Command,
    Confirm,
    Exit,
    Insert,
    Jump,
    LineJump,
    Normal,
    Open,
    Paste,
    Path,
    Search,
    Select,
    SelectLine,
    SymbolJump,
    Syntax,
    Theme,
}

pub use self::command::CommandMode;
pub use self::confirm::ConfirmMode;
pub use self::jump::JumpMode;
pub use self::line_jump::LineJumpMode;
pub use self::open::OpenMode;
pub use self::path::PathMode;
pub use self::search::SearchMode;
pub use self::search_select::{PopSearchToken, SearchSelectConfig, SearchSelectMode};
pub use self::select::SelectMode;
pub use self::select_line::SelectLineMode;
pub use self::symbol_jump::SymbolJumpMode;
pub use self::syntax::SyntaxMode;
pub use self::theme::ThemeMode;
