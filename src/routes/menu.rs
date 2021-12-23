use crate::ironpunk::*;
use crossterm::event::KeyCode;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};

pub struct MenuComponent {
    pub cid: String,
    pub selected: Option<String>,
    pub items: Vec<String>,
    pub error: Option<String>,
}
impl MenuComponent {
    pub fn new(name: &str) -> MenuComponent {
        MenuComponent {
            cid: String::from(name),
            selected: None,
            items: Vec::new(),
            error: None,
        }
    }
    pub fn index_of(&self, item: String) -> Result<usize, Error> {
        match self.items.iter().position(|i| i.clone() == item) {
            Some(pos) => Ok(pos),
            None => Err(Error::with_message(format!("invalid menu item: {}", item))),
        }
    }
    pub fn selected_index(&self) -> usize {
        match self.selected.clone() {
            Some(selected) => match self.index_of(selected) {
                Ok(index) => index,
                Err(_) => 0,
            },
            None => 0,
        }
    }
    pub fn select(&mut self, item: String) -> Result<(), Error> {
        match self.index_of(item.clone()) {
            Ok(_) => {
                self.selected = Some(item.clone());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn render_in_parent(
        &self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let menu = self
            .items
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(
                        first,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ])
            })
            .collect();
        let tabs = Tabs::new(menu)
            .select(self.selected_index())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider(Span::raw("|"));

        parent.render_widget(tabs, chunk);

        Ok(())
    }
}
impl Component for MenuComponent {
    fn name(&self) -> &str {
        "Menu"
    }
    fn id(&self) -> String {
        self.cid.clone()
    }
    #[allow(unused_variables)]
    fn process_keyboard(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool> {
        match code {
            KeyCode::Char('a') => {}
            KeyCode::Char('d') => {}
            _ => {}
        }
        Ok(false)
    }
}
