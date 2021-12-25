use crate::aes256cbc::{Config as AesConfig, Digest, Key};
use crate::core::{AUTHOR, VERSION};
use crate::ironpunk;
use crate::ironpunk::LoopEvent::*;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::{cell::RefCell, io, marker::PhantomData, rc::Rc};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
    Terminal,
};

#[allow(dead_code)]
pub struct About<'a> {
    aes_config: AesConfig,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> About<'a> {
    pub fn new(aes_config: AesConfig) -> About<'a> {
        About {
            aes_config,
            phantom: PhantomData,
        }
    }
}

impl Component for About<'_> {
    fn name(&self) -> &str {
        "About"
    }
    fn id(&self) -> String {
        String::from("About")
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        event: KeyEvent,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Esc => {
                //
                Ok(Propagate)
            }
            _ => Ok(Propagate),
        }
    }
}
impl Route for About<'_> {
    fn matches_path(&self, path: String) -> bool {
        path == String::from("/about")
    }

    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        _window: Rc<RefCell<Window>>,
    ) -> Result<(), Error> {
        terminal.draw(|rect| {
            let size = rect.size();
            let about = format!("Tomb version {} by {}", VERSION, AUTHOR);

            let footer = Paragraph::new(about)
                .style(Style::default().fg(Color::LightGreen))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("About")
                        .border_type(BorderType::Plain),
                );

            rect.render_widget(footer, size);
        })?;
        Ok(())
    }
}
