use std::fmt;
use crate::commands::Command;

// Utility type to make an Amp command function presentable (via the
// Display trait), which is required for any type used in search/select mode.
pub struct DisplayableCommand {
    pub description: &'static str,
    pub command: Command,
}

impl fmt::Display for DisplayableCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}
