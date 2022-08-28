use std::fmt;

use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;
use tui::{
    backend::Backend,
    text::{Span, Spans},
};

use crate::app::App;

use super::{state::AppState, CommandResult};

pub fn draw<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let size = rect.size();
    let main_window = draw_main();
    let mut whole_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(size.height - 1)].as_ref())
        .split(size)[1];
    rect.render_widget(main_window, size);
    whole_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(whole_chunks.width - 2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(whole_chunks)[1];
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(whole_chunks.height - 4),
                Constraint::Length(2),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(whole_chunks);

    let body = draw_body(&app.cmd_output, app.state());
    rect.render_widget(body, body_chunks[0]);
    let input = draw_input(&app.input_buffer);
    rect.render_widget(input, body_chunks[1]);
    let status = draw_status(Status::Ok("hello".to_string()));
    rect.render_widget(status, body_chunks[2]);
}

fn draw_body<'a>(file: &CommandResult, _state: &AppState) -> Paragraph<'a> {
    let tick_text: Vec<Spans> = file
        .to_string()
        .split('\n')
        .map(|s| Spans::from(format!("{}\n", s)))
        .collect();

    Paragraph::new(tick_text)
        .style(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
}

fn draw_main<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(" Cargo Function History ")
        .border_style(Style::default().fg(Color::White))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
}

fn draw_input(input: &str) -> Paragraph {
    Paragraph::new(vec![Spans::from(Span::raw(input))])
        .style(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
}

fn draw_status<'a>(status: Status) -> Paragraph<'a> {
    Paragraph::new(vec![Spans::from(Span::raw(status.to_string()))])
        .style(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
}

pub enum Status {
    Ok(String),
    Error(String),
    Warning(String),
    Loading,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok(s) => write!(f, "{}", s),
            Status::Error(s) => write!(f, "{}", s),
            Status::Warning(s) => write!(f, "{}", s),
            Status::Loading => write!(f, "Loading..."),
        }
    }
}
