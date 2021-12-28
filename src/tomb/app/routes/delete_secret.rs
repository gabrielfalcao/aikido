use super::super::components::confirmation::ConfirmationDialog;
use super::super::geometry::*;
use crate::aes256cbc::Config as AesConfig;
use crate::aes256cbc::Key;
use crate::core::version;
use crate::tomb::AES256Tomb;

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct DeleteSecret<'a> {
    key: Key,
    secret_path: Option<String>,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    phantom: PhantomData<&'a Option<()>>,
    dialog: ConfirmationDialog<'a>,
}

impl<'a> DeleteSecret<'a> {
    pub fn new(key: Key, tomb: AES256Tomb, aes_config: AesConfig) -> DeleteSecret<'a> {
        DeleteSecret {
            key,
            tomb,
            aes_config,
            secret_path: None,
            phantom: PhantomData,
            dialog: ConfirmationDialog::new(None),
        }
    }
}

impl Component for DeleteSecret<'_> {
    fn name(&self) -> &str {
        "DeleteSecret"
    }
    fn id(&self) -> String {
        match self.secret_path.clone() {
            None => panic!("DeleteSecret route did not receive the secret_path"),
            Some(path) => format!("DeleteSecret:{}", path),
        }
    }
    fn render_in_parent(
        &self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        self.dialog.render_in_parent(rect, chunk)
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        self.dialog
            .process_keyboard(event, terminal, context.clone(), router.clone())?;
        match event.code {
            KeyCode::Esc => {
                context.borrow_mut().goback();
                Ok(Propagate)
            }
            KeyCode::Left => {
                context.borrow_mut().goback();
                Ok(Propagate)
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
impl Route for DeleteSecret<'_> {}
