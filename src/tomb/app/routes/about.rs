use crate::aes256cbc::Config as AesConfig;
use crate::core::VERSION;

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
    fn render_in_parent(
        &self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let chunk = get_modal_rect(chunk);

        let version = format!("Version {}", VERSION);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green))
            .title("<press (Esc) to dismiss>")
            .border_type(BorderType::Plain);

        let containers = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(chunk);

        let top = Paragraph::new(vec![
            Spans::from(vec![Span::raw("Tomb - Password Manager")]),
            Spans::from(vec![Span::raw(&version)]),
        ])
        .style(
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
        let middle = Paragraph::new(vec![
            Spans::from(vec![Span::raw("powered by")]),
            Spans::from(vec![Span::styled(
                "AES-256-CBC",
                Style::default().fg(Color::White),
            )]),
            Spans::from(vec![Span::raw("encryption")]),
        ])
        .style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
        let bottom = Paragraph::new(vec![
            Spans::from(vec![Span::raw("Created by: Gabriel Falcão")]),
            Spans::from(vec![Span::raw("twitter: @gabrielfalcao")]),
            // Spans::from(vec![Span::raw("https://github.com/gabrielfalcao/tomb")]),
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

        rect.render_widget(block, chunk);
        rect.render_widget(top, get_padded_rect(containers[0], 2));
        rect.render_widget(middle, get_padded_rect(containers[1], 1));
        rect.render_widget(bottom, get_padded_rect(containers[2], 1));
        Ok(())
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        _router: SharedRouter,
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
impl Route for About<'_> {
    fn path(&self) -> String {
        String::from("/about")
    }

    fn matches_path(&self, path: String) -> bool {
        path.eq("/about")
    }
}
