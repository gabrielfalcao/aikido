use console;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use route_recognizer::Router;
use thiserror::Error;

use crate::logger;

use std::{
    collections::BTreeMap,
    fmt, io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

pub type Backend = CrosstermBackend<io::Stdout>;
pub type BoxedError = Box<dyn std::error::Error>;
pub type BoxedRoute = Box<dyn Route>;
pub type BoxedRoutes = Vec<BoxedRoute>;
pub type BoxedRouter = Router<BoxedRoute>;
pub type RouteMap = BTreeMap<String, BoxedRoute>;
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

#[allow(unused_variables)]
pub fn quit(terminal: &mut Terminal<Backend>) -> Result<(), Error> {
    Ok(())
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

pub use LoopEvent::*;
pub trait Component {
    fn id(&self) -> String;
    fn name(&self) -> &str;
    fn process_keyboard(
        &mut self,
        terminal: &mut Terminal<Backend>,
        event: KeyEvent,
    ) -> Result<LoopEvent, Error>;
    fn tick(&mut self, _terminal: &mut Terminal<Backend>) -> Result<LoopEvent, Error> {
        Ok(Propagate)
    }
}

pub trait Route
where
    Self: Component,
{
    fn matches_path(&self, path: String) -> bool;
    fn render(&mut self, terminal: &mut Terminal<Backend>) -> Result<(), Error>;
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
            .style(Style::default().fg(Color::White))
            .title("Error")
            .border_type(BorderType::Plain),
    )
}

pub struct ErrorRoute {
    error: Error,
}
impl ErrorRoute {
    fn new(message: String) -> ErrorRoute {
        ErrorRoute {
            error: Error::with_message(message.clone()),
        }
    }
}

impl Route for ErrorRoute {
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
    }
    fn render(&mut self, terminal: &mut Terminal<Backend>) -> Result<(), Error> {
        let paragraph = error_text(&self.error.message);

        terminal.draw(|parent| {
            let chunk = parent.size();
            parent.render_widget(paragraph, chunk);
        })?;
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
        terminal: &mut Terminal<Backend>,
        event: KeyEvent,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Char('q') => Ok(Quit),
            _ => Ok(Propagate),
        }
    }
}
#[allow(dead_code)]
pub struct Window {
    routes: BoxedRoutes,
    location: String,
    history: Vec<String>,
}

impl Window {
    pub fn from_routes(routes: BoxedRoutes) -> Window {
        Window {
            routes,
            location: String::from("/"),
            history: Vec::new(),
        }
    }
    pub fn new() -> Window {
        Window::from_routes(BoxedRoutes::new())
    }
    #[allow(unused_variables)]
    pub fn tick(&mut self, terminal: &mut Terminal<Backend>) -> Result<LoopEvent, Error> {
        for route in &mut self.routes {
            // tick every child route
            match route.tick(terminal) {
                Ok(Propagate) => {
                    // proceed to next route
                    continue;
                }
                Ok(Refresh) => {
                    // rerender and propagate
                    self.render(terminal)?;
                    return Ok(Propagate);
                }
                Ok(any) => {
                    return Ok(any);
                }
                Err(err) => return Err(Error::with_message(format!("{}", err))),
            }
        }
        Ok(Propagate)
    }
}

impl Component for Window {
    fn name(&self) -> &str {
        "Window"
    }
    fn id(&self) -> String {
        String::from("Window")
    }
    fn process_keyboard(
        &mut self,
        terminal: &mut Terminal<Backend>,
        event: KeyEvent,
    ) -> Result<LoopEvent, Error> {
        for route in self.routes.iter_mut() {
            if route.matches_path(self.location.clone()) {
                return route.process_keyboard(terminal, event);
            }
        }
        let mut error_route = ErrorRoute {
            error: match self.routes.len() == 0 {
                true => Error::with_message(format!("no routes defined")),
                false => Error::with_message(format!("undefined route: {}", self.location)),
            },
        };
        error_route.process_keyboard(terminal, event)
    }
}
impl Route for Window {
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
    }
    fn render(&mut self, terminal: &mut Terminal<Backend>) -> Result<(), Error> {
        for route in self.routes.iter_mut() {
            if route.matches_path(self.location.clone()) {
                return route.render(terminal);
            }
        }
        let mut error_route = ErrorRoute::new(match self.routes.len() == 0 {
            true => format!("no routes defined"),
            false => format!("undefined route: {}", self.location),
        });
        error_route.render(terminal)
    }
}
pub fn reset() {
    println!("\x1bc\x1b[!p\x1b[?3;4l\x1b[4l\x1b>");
}

pub fn start(routes: BoxedRoutes) -> Result<(), BoxedError> {
    reset();
    match enable_raw_mode() {
        Ok(_) => {}
        Err(error) => {
            return Err(Box::new(Error::with_message(format!(
                "cannot initialize crossterm: {}",
                error
            ))))
        }
    };

    console::set_colors_enabled(false);
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                match event::read().expect("can read events") {
                    CEvent::Key(event) => {
                        tx.send(Event::Input(event)).expect("can send events");
                    }
                    CEvent::Mouse(_event) => {}
                    CEvent::Resize(_width, _height) => {}
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
    let mut window = Window::from_routes(routes);

    loop {
        window.render(&mut terminal)?;

        match rx.recv()? {
            Event::Input(event) => {
                match window.process_keyboard(&mut terminal, event) {
                    Ok(Quit) => {
                        //Ok(return Box::new(quit(&mut terminal))),
                        disable_raw_mode()?;
                        terminal.clear()?;
                        terminal.show_cursor()?;
                        reset();
                        std::process::exit(0);
                    }
                    Ok(Propagate | Prevent) => continue,
                    Ok(Refresh) => match window.render(&mut terminal) {
                        Ok(_) => continue,
                        Err(e) => return Err(Box::new(Error::with_message(format!("{}", e)))),
                    },
                    Err(err) => return Err(Box::new(Error::with_message(format!("{}", err)))),
                };
            }
            Event::Tick => {
                match window.tick(&mut terminal) {
                    Ok(Refresh) => {
                        window.render(&mut terminal)?;
                        continue;
                    }
                    Ok(Prevent | Propagate) => continue,
                    Ok(Quit) => {
                        //Ok(return Box::new(quit(&mut terminal))),
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        terminal.clear()?;
                        std::process::exit(0);
                    }
                    Err(err) => return Err(Box::new(Error::with_message(format!("{}", err)))),
                };
            }
        }
    }
}
