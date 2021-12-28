#![allow(unused_imports)]
#![allow(dead_code)]
use crate::ironpunk::*;

use super::super::{AES256Secret, AES256Tomb};

use crate::config::YamlFile;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Sparkline, Wrap},
    Frame, Terminal,
};

#[derive(Clone)]
pub struct DeleteConfirmation {
    pub tomb: AES256Tomb,
    pub secret: AES256Secret,
    selected: usize,
}

impl DeleteConfirmation {
    #[allow(dead_code)]
    pub fn new(tomb: AES256Tomb, secret: AES256Secret) -> DeleteConfirmation {
        DeleteConfirmation {
            tomb,
            secret,
            selected: 0,
        }
    }
    fn toggle_selected(&mut self) {
        self.selected = (self.selected + 1) % 2;
    }
    fn execute(&mut self) -> Result<LoopEvent, Error> {
        Ok(Propagate)
    }
}

impl Component for DeleteConfirmation {
    fn name(&self) -> &str {
        "DeleteConfirmation"
    }
    fn id(&self) -> String {
        self.secret.path.clone()
    }
    fn render_in_parent(
        &self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let chunk = get_modal_rect(chunk);
        let confirmation = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .title(format!("Delete Secret"))
            .border_type(BorderType::Rounded);

        let (top, bottom) = vertical_split(chunk);

        let text = vec![
            Spans::from(Span::styled(
                format!("The following secret will be deleted:"),
                Style::default().fg(Color::Black),
            )),
            Spans::from(Span::styled(
                format!("{}", self.secret.path),
                Style::default().fg(Color::Red),
            )),
            Spans::from(Span::styled(
                format!("Are you sure you want to do this?"),
                Style::default().fg(Color::Black),
            )),
        ];
        let question = Paragraph::new(text)
            .block(confirmation)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let button_yes = Paragraph::new(vec![Spans::from(Span::styled(
            format!("Yes, delete"),
            match self.selected {
                0 => Style::default().bg(Color::LightRed).fg(Color::White),
                _ => Style::default().bg(Color::Red).fg(Color::White),
            },
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(match self.selected {
                    0 => Style::default().bg(Color::LightRed).fg(Color::White),
                    _ => Style::default().bg(Color::Red).fg(Color::White),
                }),
        )
        .alignment(Alignment::Center);
        let button_no = Paragraph::new(vec![Spans::from(Span::styled(
            format!("No, cancel"),
            match self.selected {
                1 => Style::default().bg(Color::LightGreen).fg(Color::White),
                _ => Style::default().bg(Color::Green).fg(Color::White),
            },
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(match self.selected {
                    1 => Style::default().bg(Color::LightGreen).fg(Color::White),

                    _ => Style::default().bg(Color::Green).fg(Color::White),
                }),
        )
        .alignment(Alignment::Center);

        let (left, right) = horizontal_split(bottom);
        parent.render_widget(question, top);
        parent.render_widget(button_yes, left);
        parent.render_widget(button_no, right);

        Ok(())
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        _router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                self.toggle_selected();
                Ok(Propagate)
            }
            KeyCode::Backspace => Ok(Propagate),
            KeyCode::Esc => Ok(Propagate),
            KeyCode::Enter => self.execute(),
            KeyCode::Char(c) => Ok(Refresh),
            _ => Ok(Propagate),
        }
    }
}

pub fn vertical_split(size: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(size);

    let top = chunks[0];
    let bottom = chunks[1];

    (top, bottom)
}

pub fn horizontal_split(size: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .margin(1)
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    let left = chunks[0];
    let right = chunks[1];

    (left, right)
}
