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
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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
pub struct App {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,

    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,

    scroll: u16,
    items: StatefulList,
}

impl App {
    fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> App {
        let items = tomb.clone().list("*").unwrap();
        App {
            key,
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

pub fn start(tomb: AES256Tomb, key: Key, aes_config: AesConfig) -> Result<(), Box<dyn Error>> {
    let app = App::new(key, tomb, aes_config);

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    // Create two chunks with equal horizontal screen space

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.process_keyboard(key.code)? {
                    return Ok(());
                }
            }
            if last_tick.elapsed() >= tick_rate {
                app.on_tick();
                last_tick = Instant::now();
            }
        }
    }
}
