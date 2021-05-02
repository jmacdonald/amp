#[cfg(not(feature = "clipboard"))]
mod dummy_clipboard;
#[cfg(not(feature = "clipboard"))]
pub use dummy_clipboard::Clipboard;

#[cfg(feature = "clipboard")]
mod system_clipboard;
#[cfg(feature = "clipboard")]
pub use system_clipboard::Clipboard;

/// In-app content can be captured in both regular and full-line selection
/// modes. This type describes the structure of said content, based on the
/// context in which it was captured. When OS-level clipboard contents are
/// used, they are always represented as inline, as we cannot infer block
/// style without the copy context.
#[derive(Debug, PartialEq)]
pub enum ClipboardContent {
    Inline(String),
    Block(String),
    None,
}

