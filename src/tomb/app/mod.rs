pub mod menu;
use crate::ironpunk;
use crate::ironpunk::*;
use menu::{dummy_paragraph, MenuComponent};

extern crate clipboard;
use super::{AES256Secret, AES256Tomb};
use crate::aes256cbc::{Config as AesConfig, Key};

#[allow(unused_imports)]
use clipboard::{ClipboardContext, ClipboardProvider};
#[allow(unused_imports)]
use mac_notification_sys::*;

use crossterm::event::KeyCode;
use std::{io, marker::PhantomData};
#[allow(unused_imports)]
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
    },
    Frame, Terminal,
};

pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<AES256Secret>,
}

impl StatefulList {
    pub fn with_items(items: Vec<AES256Secret>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn current(&mut self) -> Option<AES256Secret> {
        match self.state.selected() {
            Some(index) => Some(self.items[index].clone()),
            None => None,
        }
    }
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[allow(dead_code)]
pub struct Application<'a> {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    _s_list: PhantomData<&'a List<'a>>,
    _s_table: PhantomData<&'a Table<'a>>,

    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,
    pub menu: MenuComponent,

    pub scroll: u16,
    pub items: StatefulList,
}

impl<'a> Application<'a> {
    fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> Application<'a> {
        let items = tomb.clone().list("*").unwrap();
        let mut menu = MenuComponent::new("main-menu");
        menu.add_item("Secrets", KeyCode::Char('s')).unwrap();
        menu.add_item("Keys", KeyCode::Char('k')).unwrap();
        menu.add_item("Preferences", KeyCode::Char('p')).unwrap();

        Application {
            key,
            menu,
            tomb,
            aes_config,
            text: String::from("('s') show / (Enter or 'c') copy to clipboard"),
            label: String::from("actions"),
            visible: false,
            scroll: 0,
            error: None,
            items: StatefulList::with_items(items),
            _s_list: PhantomData,
            _s_table: PhantomData,
        }
    }

    fn render_secrets(&mut self) -> Result<(List<'a>, Table<'a>), Error> {
        let secrets = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Secrets")
            .border_type(BorderType::Plain);
        let items: Vec<_> = self
            .items
            .items
            .iter()
            .map(|secret| {
                ListItem::new(Spans::from(vec![Span::styled(
                    secret.path.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let selected_secret = match self.items.current() {
            Some(secret) => secret,
            None => match self.items.items.len() > 0 {
                true => self.items.items[0].clone(),
                false => return Err(Error::with_message(format!("no secrets to list"))),
            },
        };

        let list = List::new(items).block(secrets).highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        let secret_detail = Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(format!(
                "0x{}",
                selected_secret
                    .digest
                    .iter()
                    .map(|b| format!("{:02X}", *b))
                    .collect::<Vec::<_>>()
                    .join("")
            ))),
            Cell::from(Span::raw(selected_secret.path)),
            Cell::from(Span::raw(selected_secret.value)),
            Cell::from(Span::raw(selected_secret.updated_at.to_string())),
            Cell::from(Span::raw(selected_secret.created_at.to_string())),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "Digest",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Name",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Value",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Updated At",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Created At",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Detail")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(5),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(5),
            Constraint::Percentage(20),
        ]);

        Ok((list, secret_detail))
    }
}

impl Component for Application<'_> {
    fn name(&self) -> &str {
        "Application"
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
impl Route for Application<'_> {
    fn matches_path(&self, path: String) -> bool {
        path == String::from("/")
    }

    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Error> {
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

            let secrets_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[1]);

            match self.render_secrets() {
                Ok((left, right)) => {
                    rect.render_stateful_widget(left, secrets_chunks[0], &mut self.items.state);
                    rect.render_widget(right, secrets_chunks[1]);
                }
                Err(error) => {
                    let error = error_text(&error.message);
                    rect.render_widget(error, chunks[1]);
                }
            };

            let footer = dummy_paragraph("Footer", "Ironpunk");

            self.menu.render_in_parent(rect, chunks[0]).unwrap();
            rect.render_widget(footer, chunks[2]);
        })?;
        Ok(())
    }
}

pub fn start(
    tomb: AES256Tomb,
    key: Key,
    aes_config: AesConfig,
) -> Result<(), ironpunk::BoxedError> {
    let app = Application::new(key, tomb, aes_config);
    let mut routes = ironpunk::BoxedRoutes::new();
    let main = Box::new(app);
    routes.push(main);
    ironpunk::start(routes)
}
