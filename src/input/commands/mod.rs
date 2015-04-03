use application::Application;

pub mod application;
pub mod workspace;
pub mod buffer;
pub mod cursor;

pub struct Command {
    pub data: char,
    pub operation: fn(&mut Application),
}

impl Command {
    pub fn execute(&self, app: &mut Application) {
        (self.operation)(app);
    }
}
