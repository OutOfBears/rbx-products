use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::{Terminal, with_terminal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmState {
    Confirmed,
    Closed,
}

#[derive(Debug)]
pub struct ConfirmViewer {
    prompt: String,
    state: ConfirmState,
    should_quit: bool,
}

impl ConfirmViewer {
    fn new(prompt: String) -> Self {
        Self {
            prompt,
            should_quit: false,
            state: ConfirmState::Closed,
        }
    }

    pub fn state(&self) -> &ConfirmState {
        &self.state
    }

    pub async fn show_prompt<T: Into<String>>(prompt: T) -> ConfirmState {
        let mut backend = ratatui::init();
        let mut viewer = Self::new(prompt.into());

        with_terminal(&mut viewer, &mut backend).await;
        viewer.state().clone()
    }
}

impl Terminal for ConfirmViewer {
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let w = (area.width.min(60)).max(24);
        let h = 7u16.min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;

        let modal = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        frame.render_widget(Clear, modal);

        let block = Block::default()
            .title("Confirm Action")
            .borders(Borders::ALL);

        let yes = Span::raw("Y").style(Style::default().fg(Color::Green));
        let no = Span::raw("N").style(Style::default().fg(Color::Red));

        let text = Text::from(vec![
            Line::from(""),
            Line::from(self.prompt.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::raw("["),
                yes,
                Span::raw("]  ["),
                no,
                Span::raw("]"),
            ]),
        ]);

        let para = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(para, modal);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key_event) = event {
            if !event.is_key_press() {
                return;
            }

            match key_event.code {
                KeyCode::Char('n') => {
                    self.state = ConfirmState::Closed;
                    self.should_quit = true;
                }
                KeyCode::Char('y') => {
                    self.state = ConfirmState::Confirmed;
                    self.should_quit = true;
                }
                _ => {}
            }
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}
