use super::super::geometry::*;
pub use super::super::state::*;
use crate::ioutils::log_to_file;
use chrono::prelude::*;

pub use super::super::components::{
    menu::{dummy_paragraph, MenuComponent},
    modal::Modal,
};

use crate::ironpunk::*;

extern crate clipboard;
use super::super::{AES256Secret, AES256Tomb};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent};
use mac_notification_sys::*;
use std::{cell::RefCell, io, marker::PhantomData, rc::Rc};
use tui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Row, Table},
    Terminal,
};

const DEFAULT_PATTERN: &'static str = "*";

pub fn log(message: String) {
    log_to_file("application.log", message).unwrap()
}
#[allow(dead_code)]
pub struct Application<'a> {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    phantom: PhantomData<&'a List<'a>>,
    started_at: DateTime<Utc>,
    pattern: String,
    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,
    pub pin_visible: bool,
    pub menu: MenuComponent,
    pub overlay: Option<SharedComponent>,
    pub scroll: u16,
    pub items: StatefulList,
}

impl<'a> Application<'a> {
    pub fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> Application<'a> {
        Application {
            key,
            tomb,
            aes_config,
            menu: MenuComponent::default("Secrets"),
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
            phantom: PhantomData,
        }
    }
    pub fn set_pattern(&mut self, pattern: &str) {
        self.pattern = String::from(pattern);
    }
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    pub fn set_pinned(&mut self, pin_visible: bool) {
        self.pin_visible = pin_visible;
    }
    pub fn filter_search(&mut self, pattern: &str) {
        match self.tomb.clone().list(pattern) {
            Ok(items) => {
                self.items.update(items);
            }
            Err(err) => self.error = Some(format!("Search error: {}", err)),
        };
    }
    pub fn reset_search(&mut self) {
        self.filter_search(DEFAULT_PATTERN);
    }
    pub fn render_secrets(&mut self, pattern: String) -> Result<(List<'a>, Table<'a>), Error> {
        let secrets = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Secret")
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

        self.filter_search(&pattern);
        let selected_secret = match self.items.current() {
            Some(secret) => secret,
            None => match self.items.items.len() > 0 {
                true => self.items.items[0].clone(),
                false => {
                    return Err(Error::with_message(format!(
                        "no secrets to list using pattern: {}",
                        pattern
                    )))
                }
            },
        };

        let list = List::new(items).block(secrets).highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::White)
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
            Cell::from(Span::raw(
                chrono_humanize::HumanTime::from(selected_secret.updated_at).to_string(),
            )),
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
                "value",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "updated at",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Metadata")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ]);

        Ok((list, secret_detail))
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
    pub fn set_overlay<T: 'static + Component>(&mut self, overlay: T) {
        self.overlay = Some(Rc::new(RefCell::new(overlay)));
    }
    pub fn remove_overlay(&mut self) {
        self.overlay = None;
    }
    #[allow(dead_code)]
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error.clone());
    }
    pub fn set_label(&mut self, label: &str) {
        self.label = String::from(label);
    }
    pub fn selected_secret(&mut self) -> Result<AES256Secret, Error> {
        match self.items.current() {
            Some(secret) => Ok(secret),
            None => Err(Error::with_message(format!("no secret selected"))),
        }
    }
    pub fn get_plaintext(&mut self, secret: &AES256Secret) -> Result<String, Error> {
        match self.tomb.get_string(secret.path.as_str(), self.key.clone()) {
            Ok(secret) => Ok(secret),
            Err(err) => return Err(Error::with_message(format!("{}", err))),
        }
    }
    pub fn selected_secret_string(&mut self) -> Result<String, Error> {
        match self.selected_secret() {
            Ok(secret) => self.get_plaintext(&secret),
            Err(err) => Err(err),
        }
    }
    pub fn reset_statusbar(&mut self) {
        if !self.pin_visible {
            self.set_visible(false);
        }
        match self.selected_secret() {
            Ok(_) => {
                let label = format!("Keyboard Shortcuts");
                self.set_label(label.as_str());
                self.set_text(
                    "'t' toggle visibility / 'r' reveal secret / 'Enter' copy secret to clipboard",
                );
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
    fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match &mut self.overlay {
            Some(overlay) => {
                return overlay
                    .borrow_mut()
                    .tick(terminal, context.clone(), router.clone());
            }
            None => {}
        }
        Ok(Refresh)
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match &mut self.overlay {
            Some(overlay) => {
                if event.code == KeyCode::Esc {
                    self.remove_overlay();
                    return Ok(Propagate);
                } else {
                    return overlay.borrow_mut().process_keyboard(
                        event,
                        terminal,
                        context.clone(),
                        router.clone(),
                    );
                }
            }
            None => {}
        }
        let code = event.code;
        self.menu
            .process_keyboard(event, terminal, context.clone(), router.clone())?;
        match code {
            KeyCode::Char('q') => Ok(Quit),
            KeyCode::Char('d') => match self.items.current() {
                Some(secret) => {
                    let path = format!("/delete/{}", secret.path.clone());
                    context.borrow_mut().goto(&path);
                    Ok(Propagate)
                }
                None => Err(Error::with_message(format!(
                    "cannot delete: no secret selected"
                ))),
            },
            KeyCode::Char('a') => {
                context.borrow_mut().goto("/about");
                Ok(Refresh)
            }
            KeyCode::Char('s') => {
                self.set_pattern("*");
                Ok(Propagate)
            }
            KeyCode::Char('p') => {
                self.set_pattern("passwords/*");
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
                // TODO: context.error.clear()
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
                        context
                            .borrow_mut()
                            .error
                            .set_error(Error::with_message(format!("{}", error)));
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
    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<(), Error> {
        terminal.draw(|rect| {
            let (top, body, footer) = vertical_stack(rect.size());

            match &self.overlay {
                Some(overlay) => {
                    // TODO: render overlay as modal in the middle of the screen
                    // step 1: divide the screen into 3 vertical chunks
                    // step 2: take the middle chunk and split into 3 horizontal chunks
                    // step 3: take the middle chunk
                    let result = overlay.borrow_mut().render_in_parent(rect, body);
                    return match result {
                        Ok(_) => (),
                        Err(err) => {
                            log(format!("Overlay rendering error: {}", err));
                        }
                    };
                }
                None => {
                    let (sidebar, detail) = body_sides(body);
                    let path = context.borrow().location.clone();
                    let prefix = match router.recognize(&path) {
                        Ok(matched) => match matched.params().find("filter") {
                            Some(pattern) => String::from(pattern),
                            None => String::new(),
                        },
                        Err(err) => {
                            log(format!("route matching failed for {}: {}", path, err));
                            String::new()
                        }
                    };
                    let pattern = if prefix.len() == 0 && !path.eq("/") {
                        self.pattern.clone()
                    } else if path.eq("/") {
                        String::from(DEFAULT_PATTERN)
                    } else {
                        format!("{}*", prefix)
                    };
                    match self.render_secrets(pattern) {
                        Ok((left, right)) => {
                            rect.render_stateful_widget(left, sidebar, &mut self.items.state);
                            rect.render_widget(right, detail);
                        }
                        Err(error) => {
                            let error = error_text(
                                "Application Error",
                                "Uncaught exception:",
                                &error.message,
                            );
                            rect.render_widget(error, get_modal_rect(body));
                        }
                    };
                }
            }
            let (footer_title, footer_label) = match self.error.clone() {
                Some(error) => (error.clone(), self.text.clone()),
                None => (self.label.clone(), self.text.clone()),
            };
            let status_bar = dummy_paragraph(&footer_title, &footer_label);
            self.menu.render_in_parent(rect, top).unwrap();
            rect.render_widget(status_bar, footer);
        })?;
        Ok(())
    }
}
