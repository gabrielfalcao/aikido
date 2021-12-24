use crate::ironpunk::LoopEvent::*;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::io;
#[allow(unused_imports)]
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Widget,
        Wrap,
    },
    Frame, Terminal,
};

#[allow(dead_code)]
pub struct Overlay {
    pub title: String,
    pub text: String,
}

impl Overlay {
    #[allow(dead_code)]
    fn new(title: &str, text: &str) -> Overlay {
        Overlay {
            title: String::from(title),
            text: String::from(text),
        }
    }
    #[allow(dead_code)]
    fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
    #[allow(dead_code)]
    fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
}
impl Widget for Overlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let secrets = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Modal")
            .border_type(BorderType::Plain);
        secrets.render(area, buf);
    }
}

impl Component for Overlay {
    fn name(&self) -> &str {
        "Overlay"
    }
    fn id(&self) -> String {
        String::from("Overlay")
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        event: KeyEvent,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Char('q') => Ok(Quit),
            _ => Ok(Propagate),
        }
    }
}
