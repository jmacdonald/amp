use crate::errors::*;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

/// In-app content can be captured in both regular and full-line selection
/// modes. This type describes the structure of said content, based on the
/// context in which it was captured. When OS-level clipboard contents are
/// used, they are always represented as inline, as we cannot infer block
/// style without the copy context.
#[derive(Debug, PartialEq, Eq)]
pub enum ClipboardContent {
    Inline(String),
    Block(String),
    None,
}

impl ClipboardContent {
    pub fn text(&self) -> Option<&str> {
        match self {
            ClipboardContent::Inline(ref content) | ClipboardContent::Block(ref content) => {
                Some(content)
            }
            _ => None,
        }
    }
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
        let new_system_content = self
            .system_clipboard
            .as_mut()
            .and_then(|clip| clip.get_contents().ok())
            .filter(|con| !con.is_empty()) // treat empty content as None
            .filter(|con| Self::system_content_differs(&self.content, con))
            .map(ClipboardContent::Inline); // external content is always inline

        if let Some(content) = new_system_content {
            self.content = content;
        }

        &self.content
    }

    // Updates the in-app and system clipboards with the specified content.
    pub fn set_content(&mut self, content: ClipboardContent) -> Result<()> {
        // Update the in-app clipboard.
        self.content = content;

        if let (Some(clip), Some(text)) = (self.system_clipboard.as_mut(), self.content.text()) {
            if let Err(_) = clip.set_contents(text.to_owned()) {
                self.system_clipboard = ClipboardProvider::new()
                    .map(Some)
                    .map_err(|_| anyhow!("Failed to update or reclaim system clipboard"))?;
            }
        }

        Ok(())
    }

    fn system_content_differs(internal_content: &ClipboardContent, system_content: &str) -> bool {
        internal_content.text() != Some(system_content)
    }
}

#[cfg(test)]
mod tests {
    use super::{Clipboard, ClipboardContent};

    #[test]
    fn system_content_differs_ignores_inline_vs_block_when_text_matches() {
        assert!(!Clipboard::system_content_differs(
            &ClipboardContent::Inline("amp\n".to_string()),
            "amp\n"
        ));
        assert!(!Clipboard::system_content_differs(
            &ClipboardContent::Block("amp\n".to_string()),
            "amp\n"
        ));
    }

    #[test]
    fn system_content_differs_detects_actual_text_changes() {
        assert!(Clipboard::system_content_differs(
            &ClipboardContent::Block("amp\n".to_string()),
            "editor\n"
        ));
        assert!(Clipboard::system_content_differs(
            &ClipboardContent::None,
            "editor"
        ));
    }
}
