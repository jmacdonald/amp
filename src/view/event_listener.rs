use crate::models::application::Event;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use crate::view::Terminal;

pub struct EventListener {
    terminal: Arc<Box<dyn Terminal + Sync + Send + 'static>>,
    events: Sender<Event>,
    killswitch: Receiver<()>
}

impl EventListener {
    /// Spins up a thread that loops forever, waiting on terminal events
    /// and forwarding those to the application event channel.
    pub fn start(terminal: Arc<Box<dyn Terminal + Sync + Send + 'static>>, events: Sender<Event>, killswitch: Receiver<()>) {
        thread::spawn(move || {
            EventListener { terminal, events, killswitch }.listen();
        });
    }

    fn listen(&mut self) {
        loop {
            if let Some(event) = self.terminal.listen() {
                self.events.send(event).ok();
            } else if self.killswitch.try_recv().is_ok() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::input::Key;
    use crate::models::application::Event;
    use std::sync::mpsc;
    use super::EventListener;
    use crate::view::terminal::*;

    #[test]
    fn start_listens_for_and_sends_key_events_from_terminal() {
        let terminal = build_terminal().unwrap();
        let (event_tx, event_rx) = mpsc::channel();
        let (_, killswitch_rx) = mpsc::sync_channel(0);
        EventListener::start(terminal.clone(), event_tx, killswitch_rx);
        let event = event_rx.recv().unwrap();

        assert_eq!(event, Event::Key(Key::Char('A')));
    }
}
