use super::menu::MenuComponent;
use crate::ironpunk::*;
use crossterm::event::KeyCode;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

pub fn dummy_paragraph<'a>(title: &'a str, content: &'a str) -> Paragraph<'a> {
    Paragraph::new(content)
        .style(Style::default().fg(Color::LightGreen))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(title)
                .border_type(BorderType::Plain),
        )
}

pub struct StackedApplication {
    title: String,
    menu: MenuComponent,
}
impl StackedApplication {
    pub fn new(title: &str) -> StackedApplication {
        let mut menu = MenuComponent::new("main-menu");
        menu.add_item("Secrets", KeyCode::Char('s')).unwrap();
        menu.add_item("Keys", KeyCode::Char('k')).unwrap();
        menu.add_item("Preferences", KeyCode::Char('p')).unwrap();

        StackedApplication {
            title: String::from(title),
            menu,
        }
    }
}
impl Component for StackedApplication {
    fn name(&self) -> &str {
        "StackedApplication"
    }
    fn id(&self) -> String {
        String::from("Application")
    }
    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => Ok(true),
            code => self.menu.process_keyboard(terminal, code),
        }
    }
}

impl Route for StackedApplication {
    fn matches_path(&self, path: String) -> bool {
        path == String::from("/")
    }

    fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Error> {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let middle = dummy_paragraph("Middle", "This is the middle");
            let footer = dummy_paragraph("Footer", self.title.as_ref());

            self.menu.render_in_parent(rect, chunks[0]).unwrap();
            rect.render_widget(middle, chunks[1]);
            rect.render_widget(footer, chunks[2]);
        })?;
        Ok(())
    }
}
