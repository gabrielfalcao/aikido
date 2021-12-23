use console;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
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
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Terminal,
};

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
pub fn quit(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Error> {
    Ok(())
}

pub enum Event<I> {
    Input(I),
    Tick,
}

pub trait Component {
    // fn by_ref(self: &Self) {}
    // fn by_ref_mut(self: &mut Self) {}
    // fn by_box(self: Box<Self>) {}
    // fn by_rc(self: Rc<Self>) {}
    // fn by_arc(self: Arc<Self>) {}
    // fn by_pin(self: Pin<&Self>) {}
    // fn with_lifetime<'a>(self: &'a Self) {}
    // fn nested_pin(self: Pin<Arc<Self>>) {}

    fn id(&self) -> String;
    fn name(&self) -> &str;
    fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Error>;
    fn process_keyboard(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool>;
}

pub trait Route
where
    Self: Component,
{
    fn matches_path(&self, path: String) -> bool;
}

pub struct ErrorRoute {
    error: Error,
}
impl Route for ErrorRoute {
    #[allow(unused_variables)]
    fn matches_path(&self, path: String) -> bool {
        true
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
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => Ok(true),
            _ => Ok(false),
        }
    }
    fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Error> {
        let paragraph = Paragraph::new(vec![
            Spans::from(vec![Span::raw("Error")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(
                self.error.message.as_str(),
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
        );

        terminal.draw(|parent| {
            let chunk = parent.size();
            parent.render_widget(paragraph, chunk);
        })?;
        Ok(())
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
            routes: routes,
            location: String::from("/"),
            history: Vec::new(),
        }
    }
    pub fn new() -> Window {
        Window::from_routes(BoxedRoutes::new())
    }

    pub fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Error> {
        for route in self.routes.iter() {
            if route.matches_path(self.location.clone()) {
                return route.render(terminal);
            }
        }
        let error_route = ErrorRoute {
            error: match self.routes.len() == 0 {
                true => Error::with_message(format!("no routes defined")),
                false => Error::with_message(format!("undefined route: {}", self.location)),
            },
        };
        error_route.render(terminal)
    }
    pub fn process_keyboard(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool> {
        for route in self.routes.iter() {
            if route.matches_path(self.location.clone()) {
                return route.process_keyboard(terminal, code);
            }
        }
        let error_route = ErrorRoute {
            error: match self.routes.len() == 0 {
                true => Error::with_message(format!("no routes defined")),
                false => Error::with_message(format!("undefined route: {}", self.location)),
            },
        };
        error_route.process_keyboard(terminal, code)
    }
    #[allow(unused_variables)]
    pub fn tick(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<bool> {
        Ok(true)
    }
}

pub struct MenuComponent {
    pub cid: String,
    pub selected: Option<String>,
    pub items: Vec<String>,
    pub error: Option<String>,
}
impl MenuComponent {
    pub fn new(name: &str) -> MenuComponent {
        MenuComponent {
            cid: String::from(name),
            selected: None,
            items: Vec::new(),
            error: None,
        }
    }
    pub fn index_of(&self, item: String) -> Result<usize, Error> {
        match self.items.iter().position(|i| i.clone() == item) {
            Some(pos) => Ok(pos),
            None => Err(Error::with_message(format!("invalid menu item: {}", item))),
        }
    }
    pub fn selected_index(&self) -> usize {
        match self.selected.clone() {
            Some(selected) => match self.index_of(selected) {
                Ok(index) => index,
                Err(_) => 0,
            },
            None => 0,
        }
    }
    pub fn select(&mut self, item: String) -> Result<(), Error> {
        match self.index_of(item.clone()) {
            Ok(_) => {
                self.selected = Some(item.clone());
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
    #[allow(unused_variables)]
    fn process_keyboard(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        code: KeyCode,
    ) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => return Ok(true),

            KeyCode::Char('a') => {}
            KeyCode::Char('d') => {}
            _ => {}
        }
        Ok(false)
    }
    fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Error> {
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
        terminal.draw(|parent| {
            let chunk = parent.size();
            parent.render_widget(tabs, chunk);
        })?;
        Ok(())
    }
}

pub fn start(routes: BoxedRoutes) -> Result<(), BoxedError> {
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
    let window = Window::from_routes(routes);

    loop {
        window.render(&mut terminal)?;

        match rx.recv()? {
            Event::Input(event) => match window.process_keyboard(&mut terminal, event.code) {
                Ok(true) => {
                    //Ok(return Box::new(quit(&mut terminal))),
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    terminal.clear()?;
                    return Ok(());
                }
                Ok(false) => continue,
                Err(err) => return Err(Box::new(Error::with_message(format!("{}", err)))),
            },
            Event::Tick => match window.tick(&mut terminal) {
                Ok(value) => value,
                Err(err) => return Err(Box::new(Error::with_message(format!("{}", err)))),
            },
        };
    }
}
