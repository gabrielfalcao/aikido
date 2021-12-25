use thiserror::Error;

use crate::{ioutils::log_to_file, logger};
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::disable_raw_mode;

pub use std::{cell::RefCell, rc::Rc};
use std::{
    fmt,
    io::{self},
    marker::PhantomData,
};

use route_recognizer::Router;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

pub type Backend = CrosstermBackend<io::Stdout>;
pub type BoxedError = Box<dyn std::error::Error>;
pub type BoxedRoute = Rc<RefCell<dyn Route>>;
pub type BoxedRoutes = Vec<BoxedRoute>;
pub type BoxedRouter = Router<BoxedRoute>;

#[derive(Debug, Error, Clone)]
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
impl From<io::Error> for Error {
    fn from(input: io::Error) -> Error {
        Error::with_message(format!("{:?}", input))
    }
}
pub fn reset() {
    println!("\x1bc\x1b[!p\x1b[?3;4l\x1b[4l\x1b>");
}

pub fn exit(terminal: &mut Terminal<Backend>, code: i32) {
    disable_raw_mode().unwrap_or(());
    terminal.show_cursor().unwrap_or(());
    terminal.clear().unwrap_or(());
    reset();
    std::process::exit(code);
}
pub fn quit(terminal: &mut Terminal<Backend>) {
    exit(terminal, 1)
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub enum LoopEvent {
    Propagate,
    Prevent,
    Refresh,
    Quit,
}
pub fn log(message: String) {
    log_to_file("ironpunk.log", message).unwrap()
}
pub type BoxedContext<'a> = Rc<RefCell<Context<'a>>>;

#[derive(Clone)]
pub struct Context<'a> {
    pub location: String,
    pub history: Vec<String>,
    pub error: ErrorRoute,
    _phantom: PhantomData<&'a Context<'a>>,
}

impl<'a> Context<'_> {
    pub fn new(location: &str) -> Context<'a> {
        let location = String::from(location);
        Context {
            location: location.clone(),
            _phantom: PhantomData,
            history: vec![location],
            error: ErrorRoute::new(),
        }
    }
    pub fn goto(&mut self, location: &str) {
        let location = String::from(location);
        self.history.push(location.clone());
        self.location = location.clone();
        log(format!("goto: {}", location));
    }
    pub fn goback(&mut self) {
        if self.history.len() == 1 {
            return;
        }
        match self.history.pop() {
            Some(location) => {
                log(format!("goback: {}", location));
                self.location = location;
            }
            None => {}
        }
    }
    pub fn get_location(&self) -> String {
        self.location.clone()
    }
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }
}

pub use LoopEvent::*;

pub trait Component {
    fn id(&self) -> String;
    fn name(&self) -> &str;
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<LoopEvent, Error>;
    fn tick(
        &mut self,
        _terminal: &mut Terminal<Backend>,
        _context: BoxedContext,
    ) -> Result<LoopEvent, Error> {
        Ok(Refresh)
    }
}

pub trait Route
where
    Self: Component,
{
    fn path(&self) -> String;
    fn matches_path(&self, path: String) -> bool;
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct ErrorRoute {
    error: Option<Error>,
    title: String,
}
impl ErrorRoute {
    pub fn new_with_message(message: String) -> ErrorRoute {
        ErrorRoute {
            title: String::from("Error"),
            error: Some(Error::with_message(message.clone())),
        }
    }
    pub fn new() -> ErrorRoute {
        ErrorRoute {
            title: String::new(),
            error: None,
        }
    }
    pub fn set_error(&mut self, error: Error) {
        self.error = Some(error.clone());
    }
    pub fn set_title(&mut self, title: String) {
        self.title = title.clone();
    }
    pub fn clear(&mut self) {
        self.error = None;
    }
    pub fn exists(&self) -> bool {
        match self.error {
            Some(_) => true,
            None => false,
        }
    }
}

impl Route for ErrorRoute {
    fn path(&self) -> String {
        String::from("*")
    }
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
    }
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        _context: BoxedContext,
    ) -> Result<(), Error> {
        match &self.error {
            Some(error) => {
                let paragraph = error_text(&error.message);
                terminal.draw(|parent| {
                    let chunk = get_modal_rect(parent.size());
                    parent.render_widget(paragraph, chunk);
                })?;
            }
            None => {}
        };
        Ok(())
    }
}
impl Component for ErrorRoute {
    fn name(&self) -> &str {
        "ErrorRoute"
    }
    fn id(&self) -> String {
        String::from("Error")
    }
    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        _context: BoxedContext,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                quit(terminal);
                Ok(Quit)
            }
            _ => Ok(Propagate),
        }
    }
}

pub fn error_text<'a>(error: &'a str) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("Error")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            error,
            //console::strip_ansi_codes(self.error.message.as_str()).borrow(),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red).fg(Color::White))
            .title("Error")
            .border_type(BorderType::Plain),
    )
}

pub fn get_modal_rect(parent: Rect) -> Rect {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(parent);
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(vertical_chunks[1]);

    let center = horizontal_chunks[1];
    center
}
