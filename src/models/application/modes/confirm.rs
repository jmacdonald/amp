use crate::commands::Command;

pub struct ConfirmMode {
    pub command: Command,
}

impl ConfirmMode {
    pub fn new(command: Command) -> ConfirmMode {
        ConfirmMode { command }
    }
}
