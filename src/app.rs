use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};

use std::{
    fmt, io,
    std::sync::mpsc,
    std::time::{Duration, Instant},
    thread,
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};
pub fn quit(terminal: Terminal<dyn Backend>) {
    disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.clear()?;
    //println!("\x1bc\x1b[!p\x1b[?3;4l\x1b[4l\x1b>");
}
#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error {
    pub fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub trait Component {
    fn id(&self) -> String;
    fn name(&self) -> &str;
    fn render(&self, parent: &Rect, chunk: Rect) -> Result<(), Error>;
    fn process_keyboard(
        &mut self,
        terminal: Terminal<dyn Backend>,
        code: KeyCode,
    ) -> io::Result<bool>;
}

pub trait Route {
    fn matches_path(&self, path: &str) -> bool;
}

pub struct ErrorRoute {
    error: Error,
}
impl Component for ErrorRoute {
    fn name(&self) -> &str {
        "ErrorRoute"
    }
    fn id(&self) -> String {
        String::from("Error")
    }
    fn process_keyboard(
        &mut self,
        terminal: Terminal<dyn Backend>,
        code: KeyCode,
    ) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => {
                quit(terminal);
            }
            _ => {}
        }
    }
    fn render(&self, parent: &Rect, chunk: Rect) -> Result<(), Error> {
        let widget = Paragraph::new(vec![
            Spans::from(vec![Span::raw("Error")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(self.error.message.as_str())]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Error")
                .border_type(BorderType::Plain),
        );
        parent.render_widget(widget, chunk);
    }
}
pub struct Window {
    routes: Vec<dyn Route>,
    location: String,
    backend: dyn Backend,
    history: Vec<String>,
}

impl Window {
    pub fn new(backend: dyn Backend) -> Window {
        Window {
            routes: Vec::new(),
            location: "/",
            backend,
            history: Vec::new(),
        }
    }
    pub fn route(&self) -> dyn Route {
        if self.routes.len() == 0 {
            return ErrorRoute {
                error: Error::with_message(format!("undefined route: {}", self.location)),
            };
        }
        for route in self.routes {
            if route.matches_path(self.location) {
                return Ok(route.clone());
            }
        }
    }
}

pub struct MenuComponent {
    selected: Option<String>,
    items: Vec<String>,
    error: Option<String>,
}
impl MenuComponent {
    fn index_of(&self, item: String) -> Result<usize, Error> {
        match self.items.iter().position(|i| i == item) {
            Some(pos) => pos,
            None => Err(Error::with_message(format!("invalid menu item: {}", item))),
        }
    }
    fn selected_index(&self) -> usize {
        self.index_of(self.selected).unwrap()
    }
    fn select(&mut self, item: String) -> Result<usize, Error> {
        match self.index_of(self.selected) {
            Ok(_) => {
                self.selected = item.clone();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
impl Component for MenuComponent {
    fn name(&self) -> &str {
        "Menu"
    }
    fn id(&self) -> String {
        self.cid.clone()
    }
    fn process_keyboard(
        &mut self,
        terminal: Terminal<dyn Backend>,
        code: KeyCode,
    ) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => {
                quit(terminal);
            }
            KeyCode::Char('a') => {}
            KeyCode::Char('d') => {}
            _ => {}
        }
    }
    fn render(&self, parent: &Rect, chunk: Rect) -> Result<(), Error> {
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
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut window = Window::new(bacend);

    loop {
        let route = window.route();

        terminal.draw(|rect| {
            let size = rect.size();
            route.render(&rect, rect.clone());
        })?;

        match rx.recv()? {
            Event::Input(event) => route.process_keyboard(terminal, event.code),
            Event::Tick => {}
        }
    }

    Ok(())
}

// impl<T> Route for T
// where
//     T: Component,
// {
//     fn process_keyboard(&mut self, code: KeyCode) -> io::Result<bool> {}
//     fn render(&self, parent: &Rect, chunk: Rect) -> io::Result<bool> {}
// }
