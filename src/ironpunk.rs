use console;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use route_recognizer::Router;
use thiserror::Error;

use crate::{ioutils::log_to_file, logger};

pub use std::{cell::RefCell, rc::Rc};
use std::{
    fmt,
    io::{self},
    marker::PhantomData,
    panic,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

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
pub type BoxedContext<'a> = Rc<RefCell<Context<'a>>>;

// type ContextUpdateCallback<'a> = dyn Fn(Context);
// type BoxedContextUpdateCallback<'a> = Rc<RefCell<ContextUpdateCallback<'a>>>;

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
fn log(message: String) {
    log_to_file("ironpunk.log", message).unwrap()
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
    fn matches_path(&self, path: String) -> bool;
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<(), Error>;
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
    pub fn exists(&mut self) -> bool {
        match self.error {
            Some(_) => true,
            None => false,
        }
    }
}

impl Route for ErrorRoute {
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
            KeyCode::Esc => {
                self.clear();
                Ok(Refresh)
            }
            _ => Ok(Propagate),
        }
    }
}

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
        if self.history.len() == 0 {
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
    pub fn matches(&self, route: &dyn Route) -> bool {
        route.matches_path(self.location.clone())
    }
}

#[allow(dead_code)]
pub struct Window<'a> {
    routes: BoxedRoutes,
    context: Context<'a>,
    _phantom: PhantomData<&'a Context<'a>>,
}

// impl Clone for Window {
//     #[allow(unused_mut)]
//     fn clone(&self) -> Self {
//         let mut routes = BoxedRoutes::new();
//         // for route in &self.routes {
//         //     routes.push(Rc::clone(route));
//         // }
//         Window {
//             routes: routes,
//             history: self.history.clone(),
//             error: self.error.clone(),
//         }
//     }
//     #[allow(unused_mut)]
//     fn clone_from(&mut self, source: &Self) {
//         let mut routes = BoxedRoutes::new();
//         // for route in &source.routes {
//         //     routes.push(Rc::clone(route));
//         // }
//         self.routes = routes;
//         self.location = source.location.clone();
//         self.history = source.history.clone();
//         self.error = source.error.clone();
//     }
// }

impl<'a> Window<'a> {
    pub fn from_routes(routes: BoxedRoutes) -> Window<'a> {
        Window {
            routes,
            _phantom: PhantomData,
            context: Context::new("/"),
        }
    }
    pub fn new() -> Window<'a> {
        Window::from_routes(BoxedRoutes::new())
    }
    #[allow(unused_variables)]
    pub fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<LoopEvent, Error> {
        let context = context.borrow();
        log(format!(
            "location: {} {:?}\n",
            context.location,
            context.history.len()
        ));

        // for route in &mut self.routes.clone() {
        //     let mut route = route.borrow_mut();
        //     // tick every child route
        //     match route.tick(terminal, context.clone()) {
        //         Ok(Propagate) => {
        //             // proceed to next route
        //             continue;
        //         }
        //         Ok(Refresh) => {
        //             // rerender and propagate
        //             self.render(terminal, context.clone())?;
        //             return Ok(Refresh);
        //         }
        //         Ok(any) => {
        //             return Ok(any);
        //         }
        //         Err(err) => return Err(Error::with_message(format!("{}", err))),
        //     };
        // }
        // TODO: tick every component
        Ok(Propagate)
    }
    pub fn set_error(&mut self, error: Error) {
        self.context.error.set_error(error)
    }
    pub fn render_error(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<(), Error> {
        self.render_error(terminal, context.clone())
    }
}

impl Component for Window<'_> {
    fn name(&self) -> &str {
        "Window"
    }
    fn id(&self) -> String {
        String::from("Window")
    }
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<LoopEvent, Error> {
        for route in self.routes.iter_mut() {
            if route
                .borrow()
                .matches_path(context.borrow().location.clone())
            {
                match route
                    .borrow_mut()
                    .process_keyboard(event, terminal, context.clone())
                {
                    Err(err) => {
                        context.borrow_mut().error.set_error(err);
                        return Ok(Refresh);
                    }
                    ok => return ok,
                }
            }
        }
        if context.borrow_mut().error.exists() {
            context
                .borrow_mut()
                .error
                .process_keyboard(event, terminal, context.clone())?;
        }
        Ok(Propagate)
    }
}
impl Route for Window<'_> {
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
    }
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: BoxedContext,
    ) -> Result<(), Error> {
        for route in self.routes.iter_mut() {
            let location = context.borrow().location.clone();
            if route.borrow().matches_path(location.clone()) {
                log(format!(
                    "route {} matches {}\n",
                    route.borrow().name(),
                    location
                ));
                route.borrow_mut().render(terminal, context)?;
                return Ok(());
            }
        }
        self.set_error(Error::with_message(format!("no routes declared")));
        self.render_error(terminal, context.clone())
    }
}
pub fn reset() {
    println!("\x1bc\x1b[!p\x1b[?3;4l\x1b[4l\x1b>");
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

pub fn start(routes: BoxedRoutes) -> Result<(), BoxedError> {
    panic::set_hook(Box::new(|e| {
        disable_raw_mode().unwrap_or(());
        reset();
        logger::err::error(format!("{}", e));
    }));

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
    let tick_rate = Duration::from_millis(1000);
    // let tick_rate = Duration::from_millis(314);
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
    let context = Rc::new(RefCell::new(window.context.clone()));

    loop {
        window.render(&mut terminal, context.clone())?;

        match rx.recv()? {
            Event::Input(event) => {
                match window.process_keyboard(event, &mut terminal, context.clone()) {
                    Ok(Quit) => {
                        //Ok(return Box::new(quit(&mut terminal))),
                        disable_raw_mode()?;
                        terminal.clear()?;
                        terminal.show_cursor()?;
                        reset();
                        std::process::exit(0);
                    }
                    Ok(Propagate) => continue,
                    Ok(Prevent) => break Ok(()),
                    Ok(Refresh) => match window.render(&mut terminal, context.clone()) {
                        Ok(_) => continue,
                        Err(err) => {
                            log(format!("{}", err));

                            return Err(Box::new(Error::with_message(format!("{}", err))));
                        }
                    },
                    Err(err) => {
                        log(format!("{}", err));

                        window.set_error(err);
                        match window.render(&mut terminal, context.clone()) {
                            Ok(_) => continue,
                            Err(err) => {
                                return Err(Box::new(Error::with_message(format!("{}", err))))
                            }
                        }
                    }
                };
            }
            Event::Tick => {
                match window.tick(&mut terminal, context.clone()) {
                    Ok(Refresh) => {
                        window.render(&mut terminal, context.clone())?;
                        continue;
                    }
                    Ok(Prevent | Propagate) => continue,
                    Ok(Quit) => {
                        //Ok(return Box::new(quit(&mut terminal))),
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        terminal.clear()?;
                        reset();
                        std::process::exit(0);
                    }
                    Err(err) => return Err(Box::new(Error::with_message(format!("{}", err)))),
                };
            }
        }
    }
}
