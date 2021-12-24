use crate::ironpunk::LoopEvent::*;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame, Terminal,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Overlay {
    pub title: String,
    pub text: String,
    active: bool,
}

impl Overlay {
    #[allow(dead_code)]
    pub fn new(title: &str, text: &str) -> Overlay {
        Overlay {
            title: String::from(title),
            text: String::from(text),
            active: true,
        }
    }
    #[allow(dead_code)]
    pub fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
    #[allow(dead_code)]
    pub fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
    #[allow(dead_code)]
    pub fn write(&mut self, c: char) {
        self.text.push(c);
    }
    #[allow(dead_code)]
    pub fn backspace(&mut self) {
        self.text.pop();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    pub fn render_in_parent(
        &self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let secrets = Block::default()
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bg(Color::White)
                    .fg(Color::Black),
            )
            .title("Modal")
            .border_type(BorderType::Plain);

        parent.render_widget(secrets, chunk);

        Ok(())
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
            KeyCode::Backspace => {
                self.backspace();
                Ok(Propagate)
            }
            KeyCode::Esc => {
                self.deactivate();
                return Ok(Propagate);
            }
            KeyCode::Char(c) => {
                self.write(c);
                Ok(Refresh)
            }
            _ => Ok(Propagate),
        }
    }
}
