use crate::ironpunk::LoopEvent::*;
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
pub struct Modal {
    pub title: String,
    pub text: String,
    active: bool,
}
/// Modal with editable content
impl Modal {
    #[allow(dead_code)]
    pub fn new(title: &str, text: &str) -> Modal {
        Modal {
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
        let chunk = get_modal_rect(chunk);
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(
                Style::default()
                    .bg(Color::DarkGray)
                    .bg(Color::White)
                    .fg(Color::Black),
            )
            .title(self.title.clone())
            .border_type(BorderType::Rounded);

        let text = vec![Spans::from(Span::styled(
            self.text.clone(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ))];
        let paragraph = Paragraph::new(text)
            .block(modal)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        parent.render_widget(paragraph, chunk);

        Ok(())
    }
}

impl Component for Modal {
    fn name(&self) -> &str {
        "Modal"
    }
    fn id(&self) -> String {
        String::from("Modal")
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: Rc<RefCell<Context>>,
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
