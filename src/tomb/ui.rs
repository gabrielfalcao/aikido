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

    fn show(&mut self) {
        self.visible = true;
        match self.items.current() {
            Some(secret) => match self.tomb.get_string(secret.path.as_str(), self.key.clone()) {
                Ok(value) => {
                    self.set_label("Value");
                    self.set_text(value.as_str());
                }
                Err(error) => {
                    self.set_error(format!("{}", error));
                }
            },
            None => {}
        }
    }
    fn selected_secret_string(&mut self) -> Result<String, TombError> {
        self.visible = true;
        match self.items.current() {
            Some(secret) => self.tomb.get_string(secret.path.as_str(), self.key.clone()),
            None => Err(TombError::with_message(format!("no secret selected"))),
        }
    }
    fn process_keyboard(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('s') => self.show(),
            KeyCode::Char('c') | KeyCode::Enter => match self.items.current() {
                Some(secret) => match self.selected_secret_string() {
                    Ok(plaintext) => {
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        ctx.set_contents(plaintext).unwrap();
                        self.set_text("copied to clipboard");
                        send_notification(
                            format!("Secret {}", secret.path).as_str(),
                            &Some("copied to clipboard"),
                            "",
                            &Some("Glass"),
                        )
                        .unwrap();
                    }
                    Err(error) => {
                        self.set_error(format!("{}", error));
                    }
                },
                None => {}
            },
            KeyCode::Left => {
                self.hide();
                self.items.unselect()
            }
            KeyCode::Down => {
                self.hide();
                self.items.next()
            }
            KeyCode::Up => {
                self.hide();
                self.items.previous()
            }
            _ => {}
        }
        Ok(false)
    }
    fn hide(&mut self) {
        self.visible = false;
        self.set_text("(s) show / (c) copy to clipboard");
        self.set_label("actions");
    }
    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
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
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new(key, tomb, aes_config);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(i.path.as_str())];
            ListItem::new(lines).style(Style::default().fg(Color::Red).bg(Color::Black))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Secret"))
        .highlight_style(
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    // We can now render the item list
    let paragraph = match app.error.clone() {
        Some(err) => Paragraph::new(err.clone())
            .style(Style::default().bg(Color::Red).fg(Color::Red))
            .block(create_block(app.label.clone()))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((app.scroll, 0)),
        None => Paragraph::new(app.text.clone())
            .style(Style::default().bg(Color::Black).fg(Color::Red))
            .block(create_block(app.label.clone()))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((app.scroll, 0)),
    };
    f.render_widget(paragraph, chunks[1]);
}
