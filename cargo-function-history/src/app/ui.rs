use std::collections::BTreeMap;

use function_history_backend_thread::types::Status;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;
use tui::{
    backend::Backend,
    text::{Span, Spans},
};

use crate::app::App;

use super::{state::AppState, CommandResult};

pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let size = rect.size();
    // check if we have enough space to draw
    if size.width < 10 || size.height < 10 {
        panic!("Not enough space to draw");
    }
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
    app.get_result();
    draw_body(app, body_chunks[0], rect);
    let width = body_chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = (app.input_buffer.cursor() as u16).max(width) - width;
    let input = Paragraph::new(app.input_buffer.value())
        .style(match app.state() {
            AppState::Editing => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(
            Block::default()
                // .borders(Borders::TOP)
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
        .scroll((0, scroll));
    rect.render_widget(input, body_chunks[1]);
    if let AppState::Editing = app.state() {
        // AppState::Editing => {
        rect.set_cursor(
            // Put cursor past the end of the input text
            body_chunks[1].x + (app.input_buffer.cursor() as u16).min(width),
            // Move one line down, from the border to the input line
            body_chunks[1].y,
        )
    }
    let status = draw_status(app.status());
    rect.render_widget(status, body_chunks[2]);
}

fn draw_body<B: Backend>(app: &mut App, mut pos: Rect, frame: &mut Frame<B>) {
    let top = match &app.cmd_output {
        CommandResult::History(history) => {
            let metadata = history.get_metadata();
            let metadata = BTreeMap::from_iter(metadata.iter());
            let metadata: Vec<Spans> = metadata
                .iter()
                .map(|x| Spans::from(format!("{}: {}\n", x.0, x.1)))
                .collect();
            Some(
                Paragraph::new(metadata)
                    .style(Style::default().fg(Color::LightCyan))
                    .block(Block::default().style(Style::default().fg(Color::White))),
            )
        }
        _ => None,
    };
    let tick_text: Vec<Spans> = match &app.cmd_output {
        CommandResult::None => match app.status {
            Status::Loading => vec![Spans::from("Loading...")],
            _ => vec![Spans::from("No output")],
        },
        a => a
            .to_string()
            .split('\n')
            .map(|s| Spans::from(format!("{}\n", s)))
            .collect(),
    };
    let body = Paragraph::new(tick_text)
        .style(Style::default().fg(Color::LightCyan))
        .scroll(app.scroll_pos)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        );
    if let Some(top) = top {
        let mut top_pos = pos;
        top_pos.height = 4;
        pos.height -= 4;
        pos.y += 4;
        frame.render_widget(top, top_pos);
    }
    app.body_height = pos.height;
    frame.render_widget(body, pos);
}

fn draw_main<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(" Cargo Function History ")
        .border_style(Style::default().fg(Color::White))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
}

fn draw_status<'a>(status: &Status) -> Paragraph<'a> {
    Paragraph::new(vec![Spans::from(Span::raw(status.to_string()))])
        .style(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
}
