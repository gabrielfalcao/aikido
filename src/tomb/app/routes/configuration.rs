use crate::aes256cbc::Config as AesConfig;

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{cell::RefCell, io, marker::PhantomData, rc::Rc};
use tui::{backend::CrosstermBackend, Terminal};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Configuration<'a> {
    aes_config: AesConfig,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> Configuration<'a> {
    pub fn new(aes_config: AesConfig) -> Configuration<'a> {
        Configuration {
            aes_config,
            phantom: PhantomData,
        }
    }
}

impl Component for Configuration<'_> {
    fn name(&self) -> &str {
        "Configuration"
    }
    fn id(&self) -> String {
        String::from("Configuration")
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: Rc<RefCell<Context>>,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Esc => {
                context.borrow_mut().goto("/");
                Ok(Refresh)
            }
            _ => {
                if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('q') {
                    return Ok(Quit);
                }
                Ok(Propagate)
            }
        }
    }
}
impl Route for Configuration<'_> {
    fn path(&self) -> String {
        String::from("/configuration")
    }

    fn matches_path(&self, path: String) -> bool {
        path.eq("/configuration")
    }
}
