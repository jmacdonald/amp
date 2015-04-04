use application::Application;

pub mod application;
pub mod workspace;
pub mod buffer;
pub mod cursor;
pub mod jump_mode;

pub struct Command {
    pub data: char,
    pub operation: fn(&mut Application, &char),
}

impl Command {
    pub fn execute(&self, app: &mut Application) {
        (self.operation)(app, &self.data);
    }
}
