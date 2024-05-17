#[derive(Default)]
pub struct LineJumpMode {
    pub input: String,
}

impl LineJumpMode {
    pub fn new() -> LineJumpMode {
        LineJumpMode::default()
    }

    pub fn reset(&mut self) {
        self.input = String::new();
    }
}
