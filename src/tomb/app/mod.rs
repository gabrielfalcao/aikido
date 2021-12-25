pub mod components;
pub mod navigation;
pub mod routes;
use chrono::prelude::*;

use crate::ironpunk;
use crate::ironpunk::LoopEvent::*;
use crate::ironpunk::*;
pub use components::{
    menu::{dummy_paragraph, MenuComponent},
    modal::Modal,
};
pub use navigation::*;
use routes::*;

extern crate clipboard;
use super::{AES256Secret, AES256Tomb};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::{ClipboardContext, ClipboardProvider};
use mac_notification_sys::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::{cell::RefCell, io, marker::PhantomData, rc::Rc};
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
    pub fn empty() -> StatefulList {
        StatefulList::with_items(Vec::new())
    }

    pub fn update(&mut self, items: Vec<AES256Secret>) {
        self.items = items;
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
            Some(index) => {
                if self.items.len() < index + 1 {
                    return None;
                }
                Some(self.items[index].clone())
            }
            None => None,
        }
    }
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

const DEFAULT_PATTERN: &'static str = "*";

#[allow(dead_code)]
pub struct Application<'a> {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    _s_list: PhantomData<&'a List<'a>>,
    _s_table: PhantomData<&'a Table<'a>>,
    started_at: DateTime<Utc>,
    navigation: Navigation,
    pattern: String,
    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,
    pub pin_visible: bool,
    pub menu: MenuComponent,
    pub overlay: Option<Modal>,
    pub scroll: u16,
    pub items: StatefulList,
}

impl<'a> Application<'a> {
    fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> Application<'a> {
        let mut menu = MenuComponent::new("main-menu");
        menu.add_item("All", KeyCode::Char('a')).unwrap();
        menu.add_item("Passwords", KeyCode::Char('p')).unwrap();
        menu.add_item("Secrets", KeyCode::Char('s')).unwrap();
        menu.add_item("One-Time Passwords", KeyCode::Char('o'))
            .unwrap();

        Application {
            key,
            menu,
            tomb,
            aes_config,
            navigation: Navigation::new("/"),
            started_at: Utc::now(),
            overlay: None,
            text: String::from("Welcome to Tomb!"),
            label: String::from("actions"),
            visible: false,
            pattern: String::from(DEFAULT_PATTERN),
            pin_visible: false,
            scroll: 0,
            error: None,
            items: StatefulList::empty(),
            _s_list: PhantomData,
            _s_table: PhantomData,
        }
    }
    fn set_pattern(&mut self, pattern: &str) {
        self.pattern = String::from(pattern);
    }

    fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    fn set_pinned(&mut self, pin_visible: bool) {
        self.pin_visible = pin_visible;
    }
    fn filter_search(&mut self, pattern: &str) {
        match self.tomb.clone().list(pattern) {
            Ok(items) => {
                self.items.update(items);
            }
            Err(err) => self.error = Some(format!("Search error: {}", err)),
        };
    }
    fn reset_search(&mut self) {
        self.filter_search(DEFAULT_PATTERN);
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

        self.filter_search(self.pattern.clone().as_str());
        let selected_secret = match self.items.current() {
            Some(secret) => secret,
            None => match self.items.items.len() > 0 {
                true => self.items.items[0].clone(),
                false => return Err(Error::with_message(format!("no secrets to list"))),
            },
        };

        let list = List::new(items).block(secrets).highlight_style(
            Style::default()
                .bg(Color::Green)
                .fg(Color::LightYellow)
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
    fn set_overlay(&mut self, overlay: Modal) {
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
        if !self.pin_visible {
            self.set_visible(false);
        }
        match self.selected_secret() {
            Ok(_) => {
                let label = format!("Keyboard Shortcuts");
                self.set_label(label.as_str());
                self.set_text("'t' toggles the visibility of secrets / 'r' reveals the plaintext of the current secret / 'Enter' copies it to clipboard / 'O' shows overlay");
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
                if event.code == KeyCode::Esc {
                    self.remove_overlay();
                    return Ok(Propagate);
                } else {
                    return overlay.process_keyboard(terminal, event);
                }
            }
            None => {}
        }
        let code = event.code;
        self.menu.process_keyboard(terminal, event)?;
        match code {
            KeyCode::Char('q') => Ok(Quit),
            KeyCode::Char('k') | KeyCode::Char('K') => {
                self.set_overlay(Modal::new("Hello", "World"));
                Ok(Propagate)
            }
            KeyCode::Char('a') => {
                self.set_pattern("*");
                Ok(Propagate)
            }
            KeyCode::Char('p') => {
                self.set_pattern("passwords/*");
                Ok(Propagate)
            }
            KeyCode::Char('s') => {
                self.set_pattern("secrets/*");
                Ok(Propagate)
            }
            KeyCode::Char('o') => {
                self.set_pattern("otp/*");
                Ok(Propagate)
            }
            KeyCode::Char('r') => {
                match self.selected_secret_string() {
                    Ok(plaintext) => {
                        self.reset_statusbar();
                        self.set_visible(true);
                        self.set_pinned(false);
                    }
                    Err(error) => return Err(error),
                }
                Ok(Propagate)
            }
            KeyCode::Char('t') => {
                match self.selected_secret_string() {
                    Ok(plaintext) => {
                        self.set_pinned(true);
                        self.toggle_visible();
                        self.set_text(match self.visible {
                            true => "Secrets visible. (Press 't' again to toggle)",
                            false => "Secrets hidden. (Press 't' again to toggle)",
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
            KeyCode::Esc => {
                self.reset_search();
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
            _ => Ok(Propagate),
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
        _window: Rc<RefCell<Window>>,
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
                Some(overlay) => {
                    // TODO: render overlay as modal in the middle of the screen
                    // step 1: divide the screen into 3 vertical chunks
                    // step 2: take the middle chunk and split into 3 horizontal chunks
                    // step 3: take the middle chunk
                    let screen = chunks[1];
                    overlay.render_in_parent(rect, screen).unwrap()
                }
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
    let app = Application::new(key, tomb, aes_config.clone());
    let about = routes::about::About::new(aes_config);
    let mut routes = ironpunk::BoxedRoutes::new();
    routes.push(Box::new(app));
    routes.push(Box::new(about));
    ironpunk::start(routes)
}
