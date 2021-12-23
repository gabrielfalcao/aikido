pub mod menu;
use crate::ironpunk;
use crate::ironpunk::*;
use menu::{dummy_paragraph, MenuComponent};
extern crate clipboard;
use super::{AES256Secret, AES256Tomb, Error as TombError};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use mac_notification_sys::*;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

struct StatefulList {
    state: ListState,
    items: Vec<AES256Secret>,
}

impl StatefulList {
    fn with_items(items: Vec<AES256Secret>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn current(&mut self) -> Option<AES256Secret> {
        match self.state.selected() {
            Some(index) => Some(self.items[index].clone()),
            None => None,
        }
    }
    fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[allow(dead_code)]
pub struct Application {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,

    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,
    menu: MenuComponent,

    scroll: u16,
    items: StatefulList,
}

impl Application {
    fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> Application {
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
        }
    }

    fn on_tick(&mut self) {
        //self.events.push(event);
    }
    fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
    fn set_error(&mut self, error: String) {
        self.error = Some(error.clone());
    }
    fn set_label(&mut self, label: &str) {
        self.label = String::from(label);
    }
}

impl Component for Application {
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
impl Route for Application {
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
            let footer = dummy_paragraph("Footer", "Ironpunk");

            self.menu.render_in_parent(rect, chunks[0]).unwrap();
            rect.render_widget(middle, chunks[1]);
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
