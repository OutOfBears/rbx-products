use std::ops::{Div, Mul};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use nestify::nest;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{
    sync::products::ProductType,
    ui::{Terminal, with_terminal},
};

nest! {
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]*
    pub struct ProductDiffs  {
        pub name: String,
        pub id: u64,
        pub diffs: Vec<
            pub enum DiffChange {
                Unchanged(pub enum ProductDiff {
                    Title(String, String),
                    Description(String, String),
                    Price(u64, u64),
                    Active(bool, bool),
                }),
                Changed(ProductDiff),
                Created(ProductDiff)
            }
        >,
    }
}

#[derive(Debug)]
pub struct DiffViewer {
    view: Option<(ProductType, ProductDiffs)>,
    diffs: Vec<(ProductType, ProductDiffs)>,
    confs: Vec<(ProductType, u64)>,
    selected: usize,
    scroll: u16,
    should_quit: bool,
}

impl DiffViewer {
    fn new() -> Self {
        Self {
            should_quit: false,
            selected: 0,
            scroll: 0,
            view: None,
            diffs: vec![],
            confs: vec![],
        }
    }

    pub async fn confirm_diffs(diffs: Vec<(ProductType, ProductDiffs)>) -> Vec<(ProductType, u64)> {
        let mut backend = ratatui::init();
        let mut viewer = Self::new().with_diffs(diffs);

        with_terminal(&mut viewer, &mut backend).await;
        viewer.get_confs().clone()
    }

    pub fn get_confs(&self) -> &Vec<(ProductType, u64)> {
        &self.confs
    }

    pub fn with_diffs(mut self, diffs: Vec<(ProductType, ProductDiffs)>) -> Self {
        self.diffs = diffs;
        self.view = None;
        self
    }

    fn render_list(&mut self, area: Rect, frame: &mut Frame) {
        let tasks: Vec<ListItem> = self
            .diffs
            .iter()
            .map(|pd| {
                let confirmed = self.confs.contains(&(pd.0, pd.1.id));
                let style = if confirmed {
                    Style::default().fg(Color::White)
                } else {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(ratatui::style::Modifier::ITALIC)
                };

                let product_type = match pd.0 {
                    ProductType::GamePass => "GamePass",
                    ProductType::DevProduct => "DevProduct",
                };

                let content = vec![Line::from(format!(
                    "{} {}: {} (ID: {})",
                    if !confirmed { "*" } else { "" },
                    product_type,
                    pd.1.name,
                    pd.1.id
                ))];
                ListItem::new(content).style(style)
            })
            .collect();

        let tasks = List::new(tasks)
            .block(
                Block::default()
                    .title(" Product Diff Viewer ")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(tasks, area, &mut state);
    }

    fn render_diff(&mut self, area: Rect, frame: &mut Frame, diff: (ProductType, ProductDiffs)) {
        let chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

        let mut left_lines = vec![];
        let mut right_lines = vec![];

        for change in diff.1.diffs.iter() {
            match change {
                DiffChange::Unchanged(pd) => match pd {
                    ProductDiff::Title(old, new) => {
                        left_lines.push(Line::from(format!("  Title: {}", old)));
                        right_lines.push(Line::from(format!("  Title: {}", new)));
                    }
                    ProductDiff::Description(old, new) => {
                        left_lines.push(Line::from(format!("  Description: {}", old)));
                        right_lines.push(Line::from(format!("  Description: {}", new)));
                    }
                    ProductDiff::Price(old, new) => {
                        left_lines.push(Line::from(format!("  Price: {}", old)));
                        right_lines.push(Line::from(format!("  Price: {}", new)));
                    }
                    ProductDiff::Active(old, new) => {
                        left_lines.push(Line::from(format!("  Active: {}", old)));
                        right_lines.push(Line::from(format!("  Active: {}", new)));
                    }
                },
                DiffChange::Changed(pd) => match pd {
                    ProductDiff::Title(old, new) => {
                        left_lines.push(
                            Line::from(format!("- Title: {}", old))
                                .style(Style::default().fg(Color::Red)),
                        );
                        right_lines.push(
                            Line::from(format!("+ Title: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                    ProductDiff::Description(old, new) => {
                        left_lines.push(
                            Line::from(format!("- Description: {}", old))
                                .style(Style::default().fg(Color::Red)),
                        );
                        right_lines.push(
                            Line::from(format!("+ Description: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                    ProductDiff::Price(old, new) => {
                        left_lines.push(
                            Line::from(format!("- Price: {}", old))
                                .style(Style::default().fg(Color::Red)),
                        );
                        right_lines.push(
                            Line::from(format!("+ Price: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                    ProductDiff::Active(old, new) => {
                        left_lines.push(
                            Line::from(format!("- Active: {}", old))
                                .style(Style::default().fg(Color::Red)),
                        );
                        right_lines.push(
                            Line::from(format!("+ Active: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                },
                DiffChange::Created(pd) => match pd {
                    ProductDiff::Title(_, new) => {
                        right_lines.push(
                            Line::from(format!("+ Title: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                    ProductDiff::Description(_, new) => {
                        right_lines.push(
                            Line::from(format!("+ Description: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                    ProductDiff::Price(_, new) => {
                        right_lines.push(
                            Line::from(format!("+ Price: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                    ProductDiff::Active(_, new) => {
                        right_lines.push(
                            Line::from(format!("+ Active: {}", new))
                                .style(Style::default().fg(Color::Green)),
                        );
                    }
                },
            }
        }

        let left_paragraph = Paragraph::new(Text::from(left_lines))
            .block(
                Block::default()
                    .title(" Remote Product ")
                    .borders(Borders::ALL),
            )
            .scroll((self.scroll, 0));

        let right_paragraph = Paragraph::new(Text::from(right_lines))
            .block(
                Block::default()
                    .title(" Product Changes ")
                    .borders(Borders::ALL),
            )
            .scroll((self.scroll, 0));

        frame.render_widget(left_paragraph, chunks[0]);
        frame.render_widget(right_paragraph, chunks[1]);
    }
}

impl Terminal for DiffViewer {
    fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();
        let areas = Layout::default()
            .constraints([Constraint::Fill(100), Constraint::Min(1)].as_ref())
            .split(size);

        let body_area = areas[0];
        let keybind_area = areas[1];

        let items = vec![
            "Enter: View Diff".to_string(),
            "c: Confirm Diff".to_string(),
            "C: Confirm All Diffs".to_string(),
            "q: Quit".to_string(),
        ];

        let keybind_areas = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(
                items
                    .iter()
                    .map(|_| {
                        Constraint::Length(
                            (1.0.div((items.len() - 1) as f32) as f32).mul(100.0) as u16
                        )
                    })
                    .collect::<Vec<Constraint>>(),
            )
            .split(keybind_area);

        if let Some(diff) = &self.view {
            self.render_diff(body_area, frame, diff.clone());
        } else {
            self.render_list(body_area, frame);
        }

        for (i, item) in items.iter().enumerate() {
            let paragraph = Paragraph::new(item.clone())
                .centered()
                .block(Block::default().borders(Borders::NONE));

            frame.render_widget(paragraph, keybind_areas[i]);
        }
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key_event) = event {
            if !event.is_key_press() {
                return;
            }

            match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    if !key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        if self.view.is_some() {
                            self.view = None;
                            self.scroll = 0;
                            return;
                        }
                    }

                    self.should_quit = true;
                }
                KeyCode::Char('C') => {
                    if self.confs.len() == self.diffs.len() {
                        self.confs = vec![];
                    } else {
                        self.confs = self.diffs.iter().map(|d| (d.0, d.1.id)).collect();
                    }
                }
                KeyCode::Char('c') => {
                    if let Some(selected_diff) = self.diffs.get(self.selected) {
                        if self.confs.contains(&(selected_diff.0, selected_diff.1.id)) {
                            self.confs.retain(|&(ptype, id)| {
                                !(ptype == selected_diff.0 && id == selected_diff.1.id)
                            });
                        } else {
                            self.confs.push((selected_diff.0, selected_diff.1.id));
                        }

                        self.view = None;
                    }
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else if self.selected == 0 {
                        self.selected = self.diffs.len() - 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected + 1 < self.diffs.len() {
                        self.selected += 1;
                    } else if self.selected + 1 == self.diffs.len() {
                        self.selected = 0;
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected_diff) = self.diffs.get(self.selected) {
                        self.view = Some(selected_diff.clone());
                    }
                }
                _ => {}
            }
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}
