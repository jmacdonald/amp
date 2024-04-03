use crate::errors::*;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

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

/// Qualifies in-app copy/paste content with structural information, and
/// synchronizes said content with the OS-level clipboard (preferring it
/// in scenarios where it differs from the in-app equivalent).
pub struct Clipboard {
    content: ClipboardContent,
    system_clipboard: Option<ClipboardContext>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Clipboard {
        // Initialize and keep a reference to the system clipboard.
        let system_clipboard = match ClipboardProvider::new() {
            Ok(clipboard) => Some(clipboard),
            Err(_) => None,
        };

        Clipboard {
            content: ClipboardContent::None,
            system_clipboard,
        }
    }

    /// Returns the in-app clipboard content. However, if in-app content
    /// differs from the system clipboard, the system clipboard content will
    /// be saved to the in-app clipboard as inline data and returned instead.
    pub fn get_content(&mut self) -> &ClipboardContent {
        // Check the system clipboard for newer content.
        let new_content = match self.system_clipboard {
            Some(ref mut clipboard) => {
                match clipboard.get_contents() {
                    Ok(content) => {
                        if content.is_empty() {
                            None
                        } else {
                            // There is system clipboard content we can use.
                            match self.content {
                                ClipboardContent::Inline(ref app_content)
                                | ClipboardContent::Block(ref app_content) => {
                                    // We have in-app clipboard content, too. Prefer
                                    // the system clipboard content if they differ.
                                    if content != *app_content {
                                        Some(ClipboardContent::Inline(content))
                                    } else {
                                        None
                                    }
                                }
                                // We have no in-app clipboard content. Use the system's.
                                _ => Some(ClipboardContent::Inline(content)),
                            }
                        }
                    }
                    _ => None,
                }
            }
            None => None,
        };

        // Update the in-app clipboard if we've found newer content.
        if let Some(new_content) = new_content {
            self.content = new_content;
        }

        &self.content
    }

    // Updates the in-app and system clipboards with the specified content.
    pub fn set_content(&mut self, content: ClipboardContent) -> Result<()> {
        // Update the in-app clipboard.
        self.content = content;

        // Update the system clipboard.
        match self.content {
            ClipboardContent::Inline(ref app_content)
            | ClipboardContent::Block(ref app_content) => {
                if let Some(ref mut clipboard) = self.system_clipboard {
                    return clipboard
                        .set_contents(app_content.clone())
                        .map_err(|_| Error::from("Failed to update system clipboard"));
                }
            }
            _ => (),
        }

        Ok(())
    }
}
