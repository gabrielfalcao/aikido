use crate::ironpunk::*;

#[allow(unused_imports)]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;
#[allow(unused_imports)]
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Confirmation {
    pub title: String,
    pub text: String,
    active: bool,
}
/// Confirmation with editable content
impl Confirmation {
    #[allow(dead_code)]
    pub fn new(title: &str, text: &str) -> Confirmation {
        Confirmation {
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
}

impl Component for Confirmation {
    fn name(&self) -> &str {
        "Confirmation"
    }
    fn id(&self) -> String {
        self.text.clone()
    }
    fn render_in_parent(
        &self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let chunk = get_modal_rect(chunk);
        let confirmation = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .title(self.title.clone())
            .border_type(BorderType::Rounded);

        let paragraph_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let text = vec![Spans::from(Span::styled(
            self.text.clone(),
            paragraph_style.clone(),
        ))];
        let paragraph = Paragraph::new(text)
            .block(confirmation)
            .style(paragraph_style)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        parent.render_widget(paragraph, chunk);

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
            KeyCode::Backspace => {
                self.backspace();
                Ok(Propagate)
            }
            KeyCode::Esc => {
                self.deactivate();
                return Ok(Propagate);
            }
            KeyCode::Enter => {
                self.write('\n');
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
