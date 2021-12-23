use crate::ironpunk::*;
use crossterm::event::KeyCode;
use std::{collections::BTreeMap, io};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};

#[derive(PartialEq, Clone)]
pub struct MenuItem {
    pub label: String,
    pub code: KeyCode,
}
impl MenuItem {
    pub fn new(label: String, code: KeyCode) -> MenuItem {
        MenuItem { label, code }
    }
}

#[derive(PartialEq, Clone)]
pub struct MenuComponent {
    pub cid: String,
    pub selected: Option<usize>,
    pub labels: Vec<String>,
    pub items: BTreeMap<String, MenuItem>,
    pub error: Option<String>,
}
impl MenuComponent {
    pub fn new(name: &str) -> MenuComponent {
        MenuComponent {
            cid: String::from(name),
            selected: None,
            labels: Vec::new(),
            items: BTreeMap::new(),
            error: None,
        }
    }
    pub fn index_of(&self, item: &str) -> Result<usize, Error> {
        match self
            .labels
            .iter()
            .position(|i| i.clone() == String::from(item))
        {
            Some(pos) => Ok(pos),
            None => Err(Error::with_message(format!("invalid menu item: {}", item))),
        }
    }
    pub fn selected_index(&self) -> usize {
        match self.selected {
            Some(selected) => selected,
            None => 0,
        }
    }
    pub fn select(&mut self, item: &str) -> Result<(), Error> {
        match self.index_of(item.clone()) {
            Ok(index) => {
                self.selected = Some(index);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    pub fn add_item(&mut self, title: &str, code: KeyCode) -> Result<(), Error> {
        let label = String::from(title);
        let item = MenuItem::new(label.clone(), code);
        self.labels.push(label.clone());
        self.items.insert(label, item);
        Ok(())
    }
    pub fn remove_item(&mut self, item: &str) -> Result<(), Error> {
        match self.index_of(item) {
            Ok(index) => {
                self.labels.remove(index);
                self.items.remove(item);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
    pub fn render_in_parent(
        &self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let menu = self
            .labels
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
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool> {
        for (label, item) in &self.items {
            let label = label.clone();
            if item.code == code {
                self.select(&label);
                break;
            }
        }
        Ok(false)
    }
}
