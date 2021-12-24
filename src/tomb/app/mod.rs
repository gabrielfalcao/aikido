pub mod menu;
pub mod overlay;
use chrono::prelude::*;

use crate::ironpunk;
use crate::ironpunk::LoopEvent::*;
use crate::ironpunk::*;
pub use menu::{dummy_paragraph, MenuComponent};
pub use overlay::Overlay;

extern crate clipboard;
use super::{AES256Secret, AES256Tomb};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::{ClipboardContext, ClipboardProvider};
use mac_notification_sys::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table},
    Terminal,
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
    started_at: DateTime<Utc>,
    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,
    pub menu: MenuComponent,
    pub overlay: Option<Overlay>,
    pub scroll: u16,
    pub items: StatefulList,
}

impl<'a> Application<'a> {
    fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> Application<'a> {
        let items = tomb.clone().list("*").unwrap();
        let mut menu = MenuComponent::new("main-menu");
        menu.add_item("Secrets", KeyCode::Char('S')).unwrap();
        menu.add_item("Keys", KeyCode::Char('K')).unwrap();
        menu.add_item("Preferences", KeyCode::Char('P')).unwrap();

        Application {
            key,
            menu,
            tomb,
            aes_config,
            started_at: Utc::now(),
            overlay: None,
            text: String::from("Welcome to Tomb!"),
            label: String::from("actions"),
            visible: false,
            scroll: 0,
            error: None,
            items: StatefulList::with_items(items),
            _s_list: PhantomData,
            _s_table: PhantomData,
        }
    }

    fn toggle_visible(&mut self) {
        self.visible = !self.visible;
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

        let secret = selected_secret.clone();
        let secret_detail = Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(format!(
                "{}",
                selected_secret
                    .digest
                    .iter()
                    .map(|b| format!("{:02x}", *b))
                    .collect::<Vec::<_>>()
                    .join("")
            ))),
            Cell::from(Span::raw(selected_secret.path)),
            Cell::from(Span::raw(match self.visible {
                true => match self.get_plaintext(&secret) {
                    Ok(plaintext) => plaintext,
                    Err(err) => format!("{}", err),
                },
                false => secret.value,
            })),
            Cell::from(Span::raw(selected_secret.updated_at.to_string())),
            Cell::from(Span::raw(selected_secret.created_at.to_string())),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "digest",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "name",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "base64-encoded cyphertext",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "updated at",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "created at",
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
    fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
    fn set_overlay(&mut self, overlay: Overlay) {
        self.overlay = Some(overlay);
    }
    fn remove_overlay(&mut self) {
        self.overlay = None;
    }
    fn set_error(&mut self, error: String) {
        self.error = Some(error.clone());
    }
    fn set_label(&mut self, label: &str) {
        self.label = String::from(label);
    }
    fn selected_secret(&mut self) -> Result<AES256Secret, Error> {
        match self.items.current() {
            Some(secret) => Ok(secret),
            None => Err(Error::with_message(format!("no secret selected"))),
        }
    }
    fn get_plaintext(&mut self, secret: &AES256Secret) -> Result<String, Error> {
        match self.tomb.get_string(secret.path.as_str(), self.key.clone()) {
            Ok(secret) => Ok(secret),
            Err(err) => return Err(Error::with_message(format!("{}", err))),
        }
    }
    fn selected_secret_string(&mut self) -> Result<String, Error> {
        match self.selected_secret() {
            Ok(secret) => self.get_plaintext(&secret),
            Err(err) => Err(err),
        }
    }
    fn reset_statusbar(&mut self) {
        match self.selected_secret() {
            Ok(_) => {
                let label = format!("Keyboard Shortcuts");
                self.set_label(label.as_str());
                self.set_text("'s' shows the plaintext of the current secret / 'Enter' copies it to clipboard / 'O' shows overlay");
            }
            Err(err) => {
                let error = format!("{}", err);
                self.set_label("Error");
                self.set_text(&error);
            }
        }
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
        event: KeyEvent,
    ) -> Result<LoopEvent, Error> {
        match &mut self.overlay {
            Some(overlay) => {
                if event.code == KeyCode::Char('q') && event.modifiers == KeyModifiers::CONTROL {
                    self.remove_overlay();
                    return Ok(Propagate);
                } else {
                    return overlay.process_keyboard(terminal, event);
                }
            }
            None => {}
        }
        let code = event.code;
        match code {
            KeyCode::Char('q') => Ok(Quit),
            KeyCode::Char('O') => {
                self.set_overlay(Overlay::new("Hello", "World"));
                send_notification("Overlay Open!", &Some("success"), "", &Some("Blow")).unwrap();

                Ok(Propagate)
            }

            KeyCode::Char('s') => {
                match self.selected_secret_string() {
                    Ok(plaintext) => {
                        self.toggle_visible();
                        self.reset_statusbar();
                        self.set_text(match self.visible {
                            true => "Secrets visible. (Press 's' again to toggle)",
                            false => "Secrets hidden. (Press 's' again to toggle)",
                        });
                    }
                    Err(error) => return Err(error),
                }
                Ok(Propagate)
            }
            KeyCode::Up => {
                self.items.previous();
                self.reset_statusbar();
                Ok(Propagate)
            }
            KeyCode::Down => {
                self.items.next();
                self.reset_statusbar();
                Ok(Propagate)
            }
            KeyCode::Enter => match self.items.current() {
                Some(secret) => match self.selected_secret_string() {
                    Ok(plaintext) => {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        ctx.set_contents(plaintext).unwrap();
                        let text = format!("{} copied to clipboard", secret.path);
                        self.set_text(&text);
                        send_notification(
                            format!("Secret {}", secret.path).as_str(),
                            &Some("copied to clipboard"),
                            "",
                            &Some("Glass"),
                        )
                        .unwrap();
                        Ok(Propagate)
                    }
                    Err(error) => {
                        self.set_error(format!("{}", error));
                        Ok(Propagate)
                    }
                },
                None => Ok(Propagate),
            },
            _ => self.menu.process_keyboard(terminal, event),
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

            match &self.overlay {
                Some(overlay) => overlay.render_in_parent(rect, chunks[1]).unwrap(),
                None => {
                    let secrets_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);

                    match self.render_secrets() {
                        Ok((left, right)) => {
                            rect.render_stateful_widget(
                                left,
                                secrets_chunks[0],
                                &mut self.items.state,
                            );
                            rect.render_widget(right, secrets_chunks[1]);
                        }
                        Err(error) => {
                            let error = error_text(&error.message);
                            rect.render_widget(error, chunks[1]);
                        }
                    };
                }
            }
            let (footer_title, footer_label) = match self.error.clone() {
                Some(error) => (error.clone(), self.text.clone()),
                None => (self.label.clone(), self.text.clone()),
            };
            let footer = dummy_paragraph(&footer_title, &footer_label);
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
