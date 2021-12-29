use super::super::components::confirmation::{paragraph_style, ConfirmationDialog};

use crate::aes256cbc::Config as AesConfig;
use crate::aes256cbc::Key;

use crate::ironpunk::*;
use crate::tomb::{AES256Secret, AES256Tomb};

use super::super::logging::log_error;
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
    pub fn get_secret(
        &mut self,
        context: SharedContext,
        router: SharedRouter,
    ) -> Option<AES256Secret> {
        let path = context.borrow().location.clone();
        match router.recognize(path.as_str()) {
            Ok(matched) => {
                let params = matched.params();
                match params.find("key") {
                    Some(key) => match self.tomb.data.get(key) {
                        Some(secret) => Some(secret.clone()),
                        None => None,
                    },
                    None => None,
                }
            }
            Err(err) => {
                log_error(format!("{}", err));
                None
            }
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
        &mut self,
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
impl Route for DeleteSecret<'_> {
    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<(), Error> {
        // match self.get_secret(context.clone(), router.clone()) {
        //     Some(secret) => {
        //         self.dialog.set_question(Some(vec![
        //             Spans::from(vec![Span::raw(
        //                 "are you sure you want to delete the secret",
        //             )]),
        //             Spans::from(vec![Span::styled(
        //                 secret.path.clone(),
        //                 Style::default().fg(Color::White),
        //             )]),
        //             Spans::from(vec![Span::raw("?")]),
        //         ]));
        //     }
        //     None => {}
        // };
        self.dialog
            .set_question(Some(vec![Spans::from(vec![Span::styled(
                "are you sure you want to delete the secret",
                paragraph_style(),
            )])]));

        terminal.draw(|parent| {
            let chunk = parent.size();
            match self.render_in_parent(parent, chunk) {
                Ok(_) => (),
                Err(err) => {
                    log(format!(
                        "error rendering component {}: {}",
                        self.name(),
                        err
                    ));
                }
            }
        })?;

        Ok(())
    }
}
