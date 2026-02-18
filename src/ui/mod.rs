use std::time::Duration;

use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{DefaultTerminal, Frame};

pub mod confirm;
pub mod diffs;

const FPS: f32 = 60.0;

pub trait Terminal {
    fn render(&mut self, frame: &mut Frame);
    fn handle_event(&mut self, event: &crossterm::event::Event);
    fn should_quit(&self) -> bool;
}

pub async fn with_terminal<T>(terminal: &mut T, backend: &mut DefaultTerminal)
where
    T: Terminal,
{
    let period = Duration::from_secs_f32(1.0 / FPS);
    let mut interval = tokio::time::interval(period);
    let mut events = EventStream::new();

    while !terminal.should_quit() {
        tokio::select! {
            _ = interval.tick() => { backend.draw(|frame| terminal.render(frame)).unwrap(); },
            Some(Ok(event)) = events.next() => terminal.handle_event(&event),
        }
    }

    ratatui::restore();
}
