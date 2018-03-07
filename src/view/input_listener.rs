use models::application::Event;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use view::Terminal;

pub struct InputListener {
    terminal: Arc<Terminal + Sync + Send>,
    events: Sender<Event>
}

impl InputListener {
    /// Spins up a thread that loops forever, waiting on input from the user
    /// and forwarding key presses to the application event channel.
    pub fn start(terminal: Arc<Terminal + Sync + Send>, events: Sender<Event>) {
        thread::spawn(move || {
            InputListener {
                terminal: terminal,
                events: events
            }.listen();
        });
    }

    fn listen(&mut self) {
        loop {
            if let Some(key) = self.terminal.listen() {
                self.events.send(Event::Key(key)).ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use input::Key;
    use models::application::Event;
    use std::sync::Arc;
    use std::sync::mpsc;
    use super::InputListener;
    use view::terminal::test_terminal::TestTerminal;

    #[test]
    fn start_listens_for_and_sends_key_events_from_terminal() {
        let terminal = Arc::new(TestTerminal::new());
        let (tx, rx) = mpsc::channel();
        InputListener::start(terminal.clone(), tx);
        let event = rx.recv().unwrap();

        assert_eq!(event, Event::Key(Key::Char('A')));
    }
}
