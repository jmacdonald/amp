use crate::errors::*;
use super::ClipboardContent;

// Simple in-app clipboard without any OS interactions.
pub struct Clipboard {
    content: ClipboardContent,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Clipboard {
        Clipboard {
            content: ClipboardContent::None,
        }
    }

    // Returns the in-app clipboard content.
    pub fn get_content(&mut self) -> &ClipboardContent {
        &self.content
    }

    // Updates the in-app clipboard with the specified content.
    pub fn set_content(&mut self, content: ClipboardContent) -> Result<()> {
        self.content = content;
        Ok(())
    }
}
